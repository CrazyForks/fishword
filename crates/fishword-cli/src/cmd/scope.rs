use anyhow::{Context, Result};
use fishword_core::{deck::Deck, storage::Storage};

use crate::util::cmd_error;

/// Resolves the deck scope shared by learning commands.
///
/// An explicit `deck_id` is looked up directly and returns a protocol-shaped
/// `deck_not_found` error when missing. Without an explicit deck, this falls
/// back to the active deck and returns `Ok(None)` when no active deck is set so
/// callers can choose their command-specific `no_active_deck` behavior.
pub(crate) fn resolve_deck(
    storage: &Storage,
    deck_id: Option<i64>,
    json: bool,
) -> Result<Option<Deck>> {
    match deck_id {
        Some(id) => storage
            .get_deck_by_id(id)
            .with_context(|| format!("failed to read deck {id}"))?
            .ok_or_else(|| cmd_error(json, "deck_not_found", &format!("Deck not found: {id}")))
            .map(Some),
        None => storage
            .get_active_deck()
            .context("failed to read active deck")
            .map_err(|e| cmd_error(json, "storage_error", &e.to_string())),
    }
}
