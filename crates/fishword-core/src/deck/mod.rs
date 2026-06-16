use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    /// Catalog deck id this deck was downloaded from (e.g. via `fishword catalog fetch`),
    /// or `None` for manually created/imported decks. Used to safely identify decks that
    /// were created from the online catalog so re-fetching merges into the right deck
    /// instead of an unrelated, manually created deck with the same name.
    pub catalog_id: Option<String>,
}
