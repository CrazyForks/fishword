use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardWithState {
    pub card: Card,
    pub state: CardState,
    pub last_reviewed_at: Option<String>,
}

/// FSRS state fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    /// The card this scheduling state belongs to.
    pub card_id: i64,
    /// FSRS memory stability. Higher values mean the card can wait longer before review.
    /// New cards start at `0.0` until the first review produces a memory state.
    pub stability: f64,
    /// FSRS memory difficulty. Higher values represent cards that are harder for the user.
    /// New cards start at `0.0` until the first review produces a memory state.
    pub difficulty: f64,
    /// UTC datetime string for when this card is next due for review.
    pub due: String,
    /// Number of completed reviews for this card. `0` means the card is still new.
    pub reps: i32,
    /// Number of review attempts rated `Again`.
    pub lapses: i32,
    /// Current learning phase used by the scheduler and selector.
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

impl Rating {
    pub fn as_i64(self) -> i64 {
        self as i64
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::Again => write!(f, "again"),
            Rating::Hard => write!(f, "hard"),
            Rating::Good => write!(f, "good"),
            Rating::Easy => write!(f, "easy"),
        }
    }
}

impl FromStr for Rating {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "again" => Ok(Self::Again),
            "hard" => Ok(Self::Hard),
            "good" => Ok(Self::Good),
            "easy" => Ok(Self::Easy),
            other => Err(format!("unknown rating: {other}")),
        }
    }
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

/// 单张卡片的一条复习记录，对应 `review_log` 表中的一行。
/// 目前该表仅供 FSRS 调度器内部读取，尚未对外暴露查询接口。
/// TODO: 实现 `fishword history <word>` 命令及复习统计报表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: i64,
    pub card_id: i64,
    pub rating: Rating,
    pub reviewed_at: String,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
}
