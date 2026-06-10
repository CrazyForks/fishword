use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}
