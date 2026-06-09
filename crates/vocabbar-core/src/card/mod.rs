use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meaning {
    pub part_of_speech: String,
    pub definition: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pronunciation {
    pub notation: String,
    pub audio_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: i64,
    pub deck_id: i64,
    pub word: String,
    pub language: String,
    pub meanings: Vec<Meaning>,
    pub pronunciations: Vec<Pronunciation>,
    pub tags: Vec<String>,
    pub source: Option<Source>,
    pub created_at: String,
}

/// FSRS state fields — populated in M3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    pub card_id: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub due: String,
    pub reps: i32,
    pub lapses: i32,
    pub state: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    New,
    Learning,
    Review,
    Relearning,
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewState::New => write!(f, "new"),
            ReviewState::Learning => write!(f, "learning"),
            ReviewState::Review => write!(f, "review"),
            ReviewState::Relearning => write!(f, "relearning"),
        }
    }
}

impl std::str::FromStr for ReviewState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" => Ok(ReviewState::New),
            "learning" => Ok(ReviewState::Learning),
            "review" => Ok(ReviewState::Review),
            "relearning" => Ok(ReviewState::Relearning),
            other => Err(format!("unknown state: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i64)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl TryFrom<i64> for Rating {
    type Error = i64;
    fn try_from(v: i64) -> Result<Self, i64> {
        match v {
            1 => Ok(Rating::Again),
            2 => Ok(Rating::Hard),
            3 => Ok(Rating::Good),
            4 => Ok(Rating::Easy),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: i64,
    pub card_id: i64,
    pub rating: Rating,
    pub reviewed_at: String,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
}
