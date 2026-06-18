use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use fsrs::{MemoryState, FSRS};

use crate::{
    card::{CardState, Rating, ReviewState},
    error::{Error, Result},
    storage::Storage,
};

const DESIRED_RETENTION: f32 = 0.9;

#[derive(Debug, Clone)]
pub struct ScheduledReview {
    pub card_id: i64,
    pub rating: Rating,
    pub reviewed_at: String,
    pub due: String,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub state: ReviewState,
}

pub struct Scheduler {
    fsrs: FSRS,
    desired_retention: f32,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            fsrs: FSRS::default(),
            desired_retention: DESIRED_RETENTION,
        }
    }
}

impl Scheduler {
    pub fn review(storage: &Storage, card_id: i64, rating: Rating) -> Result<ScheduledReview> {
        Self::default().review_card(storage, card_id, rating)
    }

    pub fn next_due(
        card_state: &CardState,
        last_reviewed_at: Option<&str>,
        rating: Rating,
    ) -> Result<ScheduledReview> {
        Self::default().compute_review(card_state, last_reviewed_at, rating)
    }

    fn review_card(
        &self,
        storage: &Storage,
        card_id: i64,
        rating: Rating,
    ) -> Result<ScheduledReview> {
        let state = storage
            .get_card_state(card_id)?
            .ok_or_else(|| Error::NotFound(format!("card_state for card {card_id}")))?;
        let last_reviewed_at = storage.last_reviewed_at(card_id)?;
        let scheduled = self.compute_review(&state, last_reviewed_at.as_deref(), rating)?;
        storage.complete_review(&scheduled)?;
        Ok(scheduled)
    }

    fn compute_review(
        &self,
        card_state: &CardState,
        last_reviewed_at: Option<&str>,
        rating: Rating,
    ) -> Result<ScheduledReview> {
        let now = Utc::now();
        let previous_memory = previous_memory_state(card_state);
        let last_review = last_reviewed_at.map(parse_utc).transpose()?.unwrap_or(now);
        let elapsed_days = (now - last_review).num_days().max(0);
        let next_states = self
            .fsrs
            .next_states(
                previous_memory,
                self.desired_retention,
                elapsed_days.try_into().unwrap_or(u32::MAX),
            )
            .map_err(|error| Error::Scheduler(error.to_string()))?;
        let next = match rating {
            Rating::Again => next_states.again,
            Rating::Hard => next_states.hard,
            Rating::Good => next_states.good,
            Rating::Easy => next_states.easy,
        };

        let scheduled_days = scheduled_days(rating, next.interval);
        let due_at = if matches!(rating, Rating::Again) {
            now
        } else {
            now + Duration::days(scheduled_days.max(1))
        };
        let review_state = next_review_state(card_state, rating);

        Ok(ScheduledReview {
            card_id: card_state.card_id,
            rating,
            reviewed_at: format_utc(now),
            due: format_utc(due_at),
            elapsed_days,
            scheduled_days,
            stability: next.memory.stability as f64,
            difficulty: next.memory.difficulty as f64,
            state: review_state,
        })
    }
}

fn previous_memory_state(card_state: &CardState) -> Option<MemoryState> {
    if card_state.reps == 0 || card_state.stability <= 0.0 || card_state.difficulty <= 0.0 {
        return None;
    }
    Some(MemoryState {
        stability: card_state.stability as f32,
        difficulty: card_state.difficulty as f32,
    })
}

fn scheduled_days(rating: Rating, interval: f32) -> i64 {
    if matches!(rating, Rating::Again) {
        0
    } else {
        interval.round().max(1.0) as i64
    }
}

fn next_review_state(card_state: &CardState, rating: Rating) -> ReviewState {
    match rating {
        Rating::Again if card_state.reps == 0 => ReviewState::Learning,
        Rating::Again => ReviewState::Relearning,
        Rating::Hard if card_state.reps == 0 => ReviewState::Learning,
        Rating::Hard | Rating::Good | Rating::Easy => ReviewState::Review,
    }
}

pub(crate) fn format_utc(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn parse_utc(value: &str) -> Result<DateTime<Utc>> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return Ok(value.with_timezone(&Utc));
    }
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .map_err(|error| Error::InvalidInput(format!("invalid datetime '{value}': {error}")))?;
    Ok(DateTime::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn open_temp() -> Storage {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep().join("scheduler.db");
        Storage::open(&path).unwrap()
    }

    fn insert_card(storage: &Storage, word: &str) -> i64 {
        let deck = storage.ensure_deck("test", None).unwrap();
        storage.insert_card(deck.id, word, &[], &[]).unwrap().id
    }

    #[test]
    fn good_review_schedules_new_card_and_logs() {
        let storage = open_temp();
        let card_id = insert_card(&storage, "cancel");
        let review = Scheduler::review(&storage, card_id, Rating::Good).unwrap();
        assert!(review.scheduled_days >= 1);

        let state = storage.get_card_state(card_id).unwrap().unwrap();
        assert_eq!(state.reps, 1);
        assert!(state.stability > 0.0);
        assert!(state.difficulty > 0.0);
        assert!(matches!(state.state, ReviewState::Review));
        assert_eq!(storage.review_log_count(card_id).unwrap(), 1);
    }

    #[test]
    fn again_is_shorter_than_easy() {
        let storage = open_temp();
        let again_card = insert_card(&storage, "again");
        let easy_card = insert_card(&storage, "easy");

        let again = Scheduler::review(&storage, again_card, Rating::Again).unwrap();
        let easy = Scheduler::review(&storage, easy_card, Rating::Easy).unwrap();

        assert_eq!(again.scheduled_days, 0);
        assert!(easy.scheduled_days > again.scheduled_days);
        assert_eq!(storage.review_log_count(again_card).unwrap(), 1);
        assert_eq!(storage.review_log_count(easy_card).unwrap(), 1);
    }
}
