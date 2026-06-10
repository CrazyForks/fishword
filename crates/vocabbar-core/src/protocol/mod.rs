use serde::Serialize;

use crate::{
    card::Card,
    deck::Deck,
    scheduler::ScheduledReview,
    selector::{SelectedCard, SelectionReason},
    storage::ProgressCounts,
};

pub const CURRENT_SCHEMA: &str = "vocabbar.protocol.current.v1";
pub const NEXT_SCHEMA: &str = "vocabbar.protocol.next.v1";
pub const RATE_SCHEMA: &str = "vocabbar.protocol.rate.v1";
pub const ERROR_SCHEMA: &str = "vocabbar.protocol.error.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFormat {
    Plain,
    Compact,
    Status,
}

impl std::str::FromStr for TextFormat {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "plain" => Ok(Self::Plain),
            "compact" => Ok(Self::Compact),
            "status" => Ok(Self::Status),
            other => Err(format!("unknown format: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CardResponse {
    pub schema: &'static str,
    pub card: ProtocolCard,
    pub display: DisplayFields,
    pub progress: ProgressFields,
    pub selection: SelectionFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct RateResponse {
    pub schema: &'static str,
    pub card: ProtocolCard,
    pub display: DisplayFields,
    pub progress: ProgressFields,
    pub review: ReviewFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub schema: &'static str,
    pub error: ErrorFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorFields {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolCard {
    pub id: String,
    pub db_id: i64,
    pub term: String,
    pub language: String,
    pub phonetic: PhoneticFields,
    pub meanings: Vec<String>,
    pub deck: DeckFields,
    pub tags: Vec<String>,
    pub source: Option<SourceFields>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhoneticFields {
    pub us: Option<String>,
    pub uk: Option<String>,
    pub other: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeckFields {
    pub id: String,
    pub name: String,
    pub db_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceFields {
    pub name: String,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayFields {
    pub plain: String,
    pub compact: String,
    pub status: String,
    pub masked: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressFields {
    pub due_count: i64,
    pub new_today_remaining: i64,
    pub reviewed_today: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionFields {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewFields {
    pub rating: String,
    pub due: String,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub state: String,
}

impl CardResponse {
    pub fn current(selected: &SelectedCard, deck: &Deck, progress: ProgressCounts) -> Self {
        Self::from_selected(CURRENT_SCHEMA, selected, deck, progress)
    }

    pub fn next(selected: &SelectedCard, deck: &Deck, progress: ProgressCounts) -> Self {
        Self::from_selected(NEXT_SCHEMA, selected, deck, progress)
    }

    fn from_selected(
        schema: &'static str,
        selected: &SelectedCard,
        deck: &Deck,
        progress: ProgressCounts,
    ) -> Self {
        let card = protocol_card(&selected.card, deck);
        let display = display_fields(&selected.card, deck);
        Self {
            schema,
            card,
            display,
            progress: progress_fields(progress),
            selection: SelectionFields {
                reason: selection_reason(selected.reason).to_string(),
            },
        }
    }
}

impl RateResponse {
    pub fn new(
        card: &Card,
        deck: &Deck,
        review: &ScheduledReview,
        progress: ProgressCounts,
    ) -> Self {
        Self {
            schema: RATE_SCHEMA,
            card: protocol_card(card, deck),
            display: display_fields(card, deck),
            progress: progress_fields(progress),
            review: ReviewFields {
                rating: review.rating.to_string(),
                due: review.due.clone(),
                elapsed_days: review.elapsed_days,
                scheduled_days: review.scheduled_days,
                stability: review.stability,
                difficulty: review.difficulty,
                state: review.state.to_string(),
            },
        }
    }
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            schema: ERROR_SCHEMA,
            error: ErrorFields {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

pub fn render_card(response: &CardResponse, format: TextFormat) -> &str {
    match format {
        TextFormat::Plain => &response.display.plain,
        TextFormat::Compact => &response.display.compact,
        TextFormat::Status => &response.display.status,
    }
}

fn protocol_card(card: &Card, deck: &Deck) -> ProtocolCard {
    let phonetic = phonetic_fields(card);
    ProtocolCard {
        id: format!("{}:{}", deck.name, card.word),
        db_id: card.id,
        term: card.word.clone(),
        language: card.language.clone(),
        meanings: card
            .meanings
            .iter()
            .map(|meaning| meaning.definition.clone())
            .collect(),
        phonetic,
        deck: DeckFields {
            id: deck.name.clone(),
            name: deck
                .description
                .clone()
                .unwrap_or_else(|| deck.name.clone()),
            db_id: deck.id,
        },
        tags: card.tags.clone(),
        source: card.source.as_ref().map(|source| SourceFields {
            name: source.name.clone(),
            license: source.license.clone(),
        }),
    }
}

fn phonetic_fields(card: &Card) -> PhoneticFields {
    let mut us = None;
    let mut uk = None;
    let mut other = Vec::new();

    for pronunciation in &card.pronunciations {
        if let Some(value) = pronunciation.notation.strip_prefix("US ") {
            us = Some(value.to_string());
        } else if let Some(value) = pronunciation.notation.strip_prefix("UK ") {
            uk = Some(value.to_string());
        } else {
            other.push(pronunciation.notation.clone());
        }
    }

    PhoneticFields { us, uk, other }
}

fn display_fields(card: &Card, deck: &Deck) -> DisplayFields {
    let meanings = card
        .meanings
        .iter()
        .map(|meaning| meaning.definition.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let phonetic = display_phonetic(card);
    let plain = if phonetic.is_empty() {
        format!("{} - {}", card.word, meanings)
    } else {
        format!("{} {} - {}", card.word, phonetic, meanings)
    };
    let compact = if phonetic.is_empty() {
        format!("📚 {} {}", card.word, meanings)
    } else {
        format!("📚 {} /{}/ {}", card.word, phonetic, meanings)
    };
    let deck_name = deck.description.as_deref().unwrap_or(&deck.name);
    let status = if phonetic.is_empty() {
        format!("📚 {} | {} | {}", card.word, deck_name, meanings)
    } else {
        format!(
            "📚 {} /{}/ | {} | {}",
            card.word, phonetic, deck_name, meanings
        )
    };
    let masked = if phonetic.is_empty() {
        format!("📚 {}", mask_term(&card.word))
    } else {
        format!("📚 {} /{}/", mask_term(&card.word), phonetic)
    };
    DisplayFields {
        plain,
        compact,
        status,
        masked,
    }
}

fn display_phonetic(card: &Card) -> String {
    let fields = phonetic_fields(card);
    fields
        .us
        .or(fields.uk)
        .or_else(|| fields.other.first().cloned())
        .unwrap_or_default()
}

fn mask_term(term: &str) -> String {
    let chars = term.chars().collect::<Vec<_>>();
    match chars.as_slice() {
        [] => String::new(),
        [only] => only.to_string(),
        [first, rest @ ..] => {
            let mut masked = first.to_string();
            masked.push_str(&"_".repeat(rest.len()));
            masked
        }
    }
}

fn progress_fields(progress: ProgressCounts) -> ProgressFields {
    ProgressFields {
        due_count: progress.due_count,
        new_today_remaining: progress.new_remaining,
        reviewed_today: progress.reviewed_today,
    }
}

fn selection_reason(reason: SelectionReason) -> &'static str {
    match reason {
        SelectionReason::Due => "due",
        SelectionReason::New => "new",
        SelectionReason::Mature => "mature",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        card::{Meaning, Pronunciation, Source},
        selector::SelectedCard,
    };

    fn sample_card() -> Card {
        Card {
            id: 1,
            deck_id: 1,
            word: "cancel".to_string(),
            language: "en".to_string(),
            meanings: vec![Meaning {
                part_of_speech: "".to_string(),
                definition: "取消，撤销；删去".to_string(),
                example: None,
            }],
            pronunciations: vec![
                Pronunciation {
                    notation: "US 'kænsl".to_string(),
                    audio_url: None,
                },
                Pronunciation {
                    notation: "UK 'kænsl".to_string(),
                    audio_url: None,
                },
            ],
            tags: vec!["cet4".to_string()],
            source: Some(Source {
                name: "qwerty-learner".to_string(),
                license: Some("GPL-3.0".to_string()),
            }),
            created_at: "2026-06-10 00:00:00".to_string(),
        }
    }

    fn sample_deck() -> Deck {
        Deck {
            id: 1,
            name: "cet4".to_string(),
            description: Some("CET-4".to_string()),
            created_at: "2026-06-10 00:00:00".to_string(),
        }
    }

    #[test]
    fn current_response_serializes_stable_fields() {
        let selected = SelectedCard {
            card: sample_card(),
            reason: SelectionReason::New,
        };
        let response = CardResponse::current(
            &selected,
            &sample_deck(),
            ProgressCounts {
                due_count: 1,
                new_remaining: 8,
                reviewed_today: 5,
            },
        );
        let value = serde_json::to_value(response).unwrap();
        let fixture = serde_json::from_str::<serde_json::Value>(include_str!(
            "../../fixtures/protocol_current_sample.json"
        ))
        .unwrap();
        assert_eq!(value, fixture);
    }

    #[test]
    fn error_response_serializes() {
        let value = serde_json::to_value(ErrorResponse::new(
            "no_cards",
            "No cards found. Import a deck first.",
        ))
        .unwrap();
        let fixture = serde_json::from_str::<serde_json::Value>(include_str!(
            "../../fixtures/protocol_error_sample.json"
        ))
        .unwrap();
        assert_eq!(value, fixture);
    }

    #[test]
    fn protocol_schema_is_valid_json() {
        serde_json::from_str::<serde_json::Value>(include_str!(
            "../../../../schemas/protocol.v1.schema.json"
        ))
        .unwrap();
    }
}
