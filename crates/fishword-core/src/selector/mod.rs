use chrono::Utc;

use crate::{
    card::{Card, CardWithState},
    error::Result,
    scheduler::parse_utc,
    storage::Storage,
};

const DEFAULT_DAILY_NEW_LIMIT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReason {
    Due,
    New,
    Mature,
}

#[derive(Debug, Clone)]
pub struct SelectedCard {
    pub card: Card,
    pub reason: SelectionReason,
}

pub struct Selector {
    daily_new_limit: usize,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            daily_new_limit: DEFAULT_DAILY_NEW_LIMIT,
        }
    }
}

impl Selector {
    pub fn select_current(storage: &Storage) -> Result<Option<SelectedCard>> {
        if let Some(card) = storage.get_current_card()? {
            return Ok(Some(SelectedCard {
                card,
                reason: SelectionReason::Mature,
            }));
        }
        Self::select_next(storage)
    }

    pub fn select_current_in_deck(storage: &Storage, deck_id: i64) -> Result<Option<SelectedCard>> {
        if let Some(card) = storage.get_current_card_in_deck(deck_id)? {
            return Ok(Some(SelectedCard {
                card,
                reason: SelectionReason::Mature,
            }));
        }
        Self::select_next_in_deck(storage, deck_id)
    }

    pub fn select_next(storage: &Storage) -> Result<Option<SelectedCard>> {
        let selector = Self::default();
        let current_card_id = storage.get_current_card_id()?;
        let candidates = storage.list_cards_with_state()?;
        let selected = selector.select_from_candidates(&candidates, current_card_id)?;
        if let Some(selected) = &selected {
            storage.set_current_card_id(Some(selected.card.id))?;
        }
        Ok(selected)
    }

    pub fn select_next_in_deck(storage: &Storage, deck_id: i64) -> Result<Option<SelectedCard>> {
        let selector = Self::default();
        let current_card_id = storage
            .get_current_card_in_deck(deck_id)?
            .map(|card| card.id);
        let candidates = storage.list_cards_with_state_by_deck(deck_id)?;
        let selected = selector.select_from_candidates(&candidates, current_card_id)?;
        if let Some(selected) = &selected {
            storage.set_current_card_id(Some(selected.card.id))?;
        }
        Ok(selected)
    }

    pub fn select_from_candidates(
        &self,
        candidates: &[CardWithState],
        skip_card_id: Option<i64>,
    ) -> Result<Option<SelectedCard>> {
        let all_candidates = candidates.iter().collect::<Vec<_>>();
        let filtered_candidates = candidates
            .iter()
            .filter(|candidate| Some(candidate.card.id) != skip_card_id)
            .collect::<Vec<_>>();
        let candidates = if filtered_candidates.is_empty() {
            // If there is only one selectable card, `next` keeps it selected.
            // Otherwise the command would turn a valid single-card deck into no card.
            all_candidates
        } else {
            filtered_candidates
        };
        let now = Utc::now();
        let mut due = candidates
            .iter()
            .filter(|candidate| candidate.state.reps > 0)
            .filter_map(|candidate| {
                let due_at = parse_utc(&candidate.state.due).ok()?;
                (due_at <= now).then_some(candidate)
            })
            .collect::<Vec<_>>();
        due.sort_by(|left, right| {
            retrievability(left)
                .partial_cmp(&retrievability(right))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.state.due.cmp(&right.state.due))
                .then_with(|| left.card.id.cmp(&right.card.id))
        });
        if let Some(candidate) = due.first() {
            return Ok(Some(selected(candidate, SelectionReason::Due)));
        }

        let new_seen = candidates
            .iter()
            .filter(|candidate| candidate.state.reps == 0)
            .take(self.daily_new_limit)
            .collect::<Vec<_>>();
        if let Some(candidate) = new_seen.first() {
            return Ok(Some(selected(candidate, SelectionReason::New)));
        }

        let mut mature = candidates
            .iter()
            .filter(|candidate| candidate.state.reps > 0)
            .collect::<Vec<_>>();
        mature.sort_by(|left, right| {
            left.state
                .due
                .cmp(&right.state.due)
                .then_with(|| left.card.id.cmp(&right.card.id))
        });
        Ok(mature
            .first()
            .map(|candidate| selected(candidate, SelectionReason::Mature)))
    }
}

fn selected(candidate: &CardWithState, reason: SelectionReason) -> SelectedCard {
    SelectedCard {
        card: candidate.card.clone(),
        reason,
    }
}

fn retrievability(candidate: &CardWithState) -> f64 {
    if candidate.state.stability <= 0.0 {
        return 0.0;
    }
    let Some(last_reviewed_at) = candidate.last_reviewed_at.as_deref() else {
        return 0.0;
    };
    let Ok(last_reviewed_at) = parse_utc(last_reviewed_at) else {
        return 0.0;
    };
    let elapsed_days = (Utc::now() - last_reviewed_at).num_days().max(0) as f64;
    (-elapsed_days / candidate.state.stability).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{card::Rating, scheduler::Scheduler, storage::Storage};

    fn open_temp() -> Storage {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep().join("selector.db");
        Storage::open(&path).unwrap()
    }

    fn insert_card(storage: &Storage, word: &str) -> i64 {
        let deck = storage.ensure_deck("test", None).unwrap();
        storage.insert_card(deck.id, word, &[], &[]).unwrap().id
    }

    fn insert_card_in_deck(storage: &Storage, deck_name: &str, word: &str) -> i64 {
        let deck = storage.ensure_deck(deck_name, None).unwrap();
        storage.insert_card(deck.id, word, &[], &[]).unwrap().id
    }

    #[test]
    fn next_selects_new_card_without_review_log() {
        let storage = open_temp();
        let card_id = insert_card(&storage, "cancel");
        let selected = Selector::select_next(&storage).unwrap().unwrap();
        assert_eq!(selected.card.id, card_id);
        assert_eq!(storage.review_log_count(card_id).unwrap(), 0);
        assert_eq!(storage.get_current_card_id().unwrap(), Some(card_id));
    }

    #[test]
    fn next_skips_current_card_when_possible() {
        let storage = open_temp();
        let first_id = insert_card(&storage, "first");
        let second_id = insert_card(&storage, "second");

        storage.set_current_card_id(Some(first_id)).unwrap();
        let selected = Selector::select_next(&storage).unwrap().unwrap();

        assert_eq!(selected.card.id, second_id);
        assert_eq!(storage.review_log_count(first_id).unwrap(), 0);
        assert_eq!(storage.review_log_count(second_id).unwrap(), 0);
    }

    #[test]
    fn due_review_card_beats_new_card() {
        let storage = open_temp();
        let due_card = insert_card(&storage, "due");
        insert_card(&storage, "new");

        Scheduler::review(&storage, due_card, Rating::Again).unwrap();
        storage.set_current_card_id(None).unwrap();
        let selected = Selector::select_next(&storage).unwrap().unwrap();

        assert_eq!(selected.card.id, due_card);
        assert_eq!(selected.reason, SelectionReason::Due);
        assert_eq!(storage.review_log_count(due_card).unwrap(), 1);
    }

    #[test]
    fn mature_card_is_fallback_when_no_new_or_due_cards() {
        let storage = open_temp();
        let card_id = insert_card(&storage, "known");
        Scheduler::review(&storage, card_id, Rating::Easy).unwrap();

        let selected = Selector::select_next(&storage).unwrap().unwrap();
        assert_eq!(selected.card.id, card_id);
        assert_eq!(selected.reason, SelectionReason::Mature);
    }

    #[test]
    fn next_in_deck_only_selects_candidates_from_that_deck() {
        let storage = open_temp();
        let first_id = insert_card_in_deck(&storage, "first", "same");
        let second_id = insert_card_in_deck(&storage, "second", "same");
        let second_deck = storage.get_deck_by_name("second").unwrap().unwrap();

        let selected = Selector::select_next_in_deck(&storage, second_deck.id)
            .unwrap()
            .unwrap();

        assert_eq!(selected.card.id, second_id);
        assert_ne!(selected.card.id, first_id);
    }

    #[test]
    fn current_in_deck_ignores_current_card_from_other_deck() {
        let storage = open_temp();
        let first_id = insert_card_in_deck(&storage, "first", "first");
        let second_id = insert_card_in_deck(&storage, "second", "second");
        let second_deck = storage.get_deck_by_name("second").unwrap().unwrap();

        storage.set_current_card_id(Some(first_id)).unwrap();
        let selected = Selector::select_current_in_deck(&storage, second_deck.id)
            .unwrap()
            .unwrap();

        assert_eq!(selected.card.id, second_id);
    }
}
