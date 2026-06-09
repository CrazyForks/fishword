use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

use crate::{
    card::{Card, CardState, Meaning, Pronunciation, ReviewState},
    deck::Deck,
    error::{Error, Result},
};

const MIGRATION: &str = include_str!("../../../../migrations/0001_init.sql");

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open (or create) the database at `path` and run migrations.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        conn.execute_batch(MIGRATION)?;
        Ok(Self { conn })
    }

    /// Platform-appropriate default database path.
    ///
    /// - macOS:   `~/Library/Application Support/vocabbar/vocabbar.db`
    /// - Linux:   `~/.local/share/vocabbar/vocabbar.db`
    /// - Windows: `%APPDATA%\vocabbar\vocabbar.db`
    pub fn default_path() -> Result<PathBuf> {
        let base = dirs::data_dir().ok_or(Error::NoDataDir)?;
        Ok(base.join("vocabbar").join("vocabbar.db"))
    }

    // ── Deck ──────────────────────────────────────────────────────────────

    pub fn list_decks(&self) -> Result<Vec<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at FROM decks ORDER BY created_at",
        )?;
        let decks = stmt
            .query_map([], |row| {
                Ok(Deck {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(decks)
    }

    pub fn insert_deck(&self, name: &str, description: Option<&str>) -> Result<Deck> {
        self.conn.execute(
            "INSERT INTO decks (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;
        let id = self.conn.last_insert_rowid();
        let deck = self.conn.query_row(
            "SELECT id, name, description, created_at FROM decks WHERE id = ?1",
            params![id],
            |row| {
                Ok(Deck {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )?;
        Ok(deck)
    }

    // ── Card ──────────────────────────────────────────────────────────────

    pub fn list_cards_by_deck(&self, deck_name: &str) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.word, c.meanings, c.pronunciations, c.created_at
             FROM cards c
             JOIN decks d ON c.deck_id = d.id
             WHERE d.name = ?1
             ORDER BY c.created_at",
        )?;
        let rows = stmt
            .query_map(params![deck_name], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        rows.into_iter()
            .map(|(id, deck_id, word, meanings_json, pronunciations_json, created_at)| {
                Ok(Card {
                    id,
                    deck_id,
                    word,
                    meanings: serde_json::from_str(&meanings_json)?,
                    pronunciations: serde_json::from_str(&pronunciations_json)?,
                    created_at,
                })
            })
            .collect()
    }

    pub fn insert_card(
        &self,
        deck_id: i64,
        word: &str,
        meanings: &[Meaning],
        pronunciations: &[Pronunciation],
    ) -> Result<Card> {
        let meanings_json = serde_json::to_string(meanings)?;
        let pronunciations_json = serde_json::to_string(pronunciations)?;
        self.conn.execute(
            "INSERT INTO cards (deck_id, word, meanings, pronunciations) VALUES (?1, ?2, ?3, ?4)",
            params![deck_id, word, meanings_json, pronunciations_json],
        )?;
        let id = self.conn.last_insert_rowid();

        // Insert a default card_state row.
        self.conn.execute(
            "INSERT OR IGNORE INTO card_state (card_id) VALUES (?1)",
            params![id],
        )?;

        let card = self.conn.query_row(
            "SELECT id, deck_id, word, meanings, pronunciations, created_at FROM cards WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )?;

        Ok(Card {
            id: card.0,
            deck_id: card.1,
            word: card.2,
            meanings: serde_json::from_str(&card.3)?,
            pronunciations: serde_json::from_str(&card.4)?,
            created_at: card.5,
        })
    }

    /// Fetch the FSRS state for a card (returns `None` if card has no state row).
    pub fn get_card_state(&self, card_id: i64) -> Result<Option<CardState>> {
        let result = self.conn.query_row(
            "SELECT card_id, stability, difficulty, due, reps, lapses, state
             FROM card_state WHERE card_id = ?1",
            params![card_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        );

        match result {
            Ok((card_id, stability, difficulty, due, reps, lapses, state_str)) => {
                let state = state_str
                    .parse::<ReviewState>()
                    .unwrap_or(ReviewState::New);
                Ok(Some(CardState {
                    card_id,
                    stability,
                    difficulty,
                    due,
                    reps,
                    lapses,
                    state,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> Storage {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep().join("test.db");
        Storage::open(&path).unwrap()
    }

    #[test]
    fn test_migration_runs() {
        open_temp();
    }

    #[test]
    fn test_deck_insert_and_list() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", Some("CET-4 vocabulary")).unwrap();
        assert_eq!(deck.name, "cet4");
        assert_eq!(deck.description.as_deref(), Some("CET-4 vocabulary"));

        let decks = storage.list_decks().unwrap();
        assert_eq!(decks.len(), 1);
        assert_eq!(decks[0].name, "cet4");
    }

    #[test]
    fn test_card_insert_and_list() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", None).unwrap();

        let meanings = vec![Meaning {
            part_of_speech: "v.".to_string(),
            definition: "取消，撤销".to_string(),
            example: Some("cancel a meeting".to_string()),
        }];
        let pronunciations = vec![Pronunciation {
            notation: "/ˈkænsl/".to_string(),
            audio_url: None,
        }];
        let card = storage
            .insert_card(deck.id, "cancel", &meanings, &pronunciations)
            .unwrap();
        assert_eq!(card.word, "cancel");
        assert_eq!(card.meanings.len(), 1);
        assert_eq!(card.meanings[0].definition, "取消，撤销");

        let cards = storage.list_cards_by_deck("cet4").unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].word, "cancel");
        assert_eq!(cards[0].pronunciations[0].notation, "/ˈkænsl/");
    }

    #[test]
    fn test_card_state_created_on_insert() {
        let storage = open_temp();
        let deck = storage.insert_deck("test", None).unwrap();
        let card = storage
            .insert_card(deck.id, "hello", &[], &[])
            .unwrap();
        let state = storage.get_card_state(card.id).unwrap();
        assert!(state.is_some());
        assert!(matches!(state.unwrap().state, ReviewState::New));
    }

    #[test]
    fn test_list_cards_empty_deck() {
        let storage = open_temp();
        storage.insert_deck("empty", None).unwrap();
        let cards = storage.list_cards_by_deck("empty").unwrap();
        assert!(cards.is_empty());
    }

    #[test]
    fn test_default_path_is_some() {
        // Just ensure it doesn't error on this platform.
        let path = Storage::default_path().unwrap();
        assert!(path.to_str().unwrap().contains("vocabbar"));
    }
}
