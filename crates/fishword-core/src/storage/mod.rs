use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

use crate::{
    card::{Card, CardState, CardWithState, Meaning, Pronunciation, ReviewState, Source},
    deck::Deck,
    error::{Error, Result},
    importer::{DuplicateStrategy, ImportCard, ImportSummary},
    scheduler::ScheduledReview,
};

const MIGRATION: &str = include_str!("../../../../migrations/0001_init.sql");

pub struct Storage {
    conn: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressCounts {
    pub due_count: i64,
    pub new_remaining: i64,
    pub reviewed_today: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyReviewStats {
    pub reviews: i64,
    pub again: i64,
    pub hard: i64,
    pub good: i64,
    pub easy: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyReviewBucket {
    pub date: String,
    pub stats: DailyReviewStats,
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
        ensure_card_metadata_columns(&conn)?;
        Ok(Self { conn })
    }

    /// Platform-appropriate default database path.
    ///
    /// - macOS:   `~/Library/Application Support/fishword/fishword.db`
    /// - Linux:   `~/.local/share/fishword/fishword.db`
    /// - Windows: `%APPDATA%\fishword\fishword.db`
    pub fn default_path() -> Result<PathBuf> {
        let base = dirs::data_dir().ok_or(Error::NoDataDir)?;
        Ok(base.join("fishword").join("fishword.db"))
    }

    // ── Deck ──────────────────────────────────────────────────────────────

    pub fn list_decks(&self) -> Result<Vec<Deck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at FROM decks ORDER BY created_at")?;
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

    pub fn get_active_deck_id(&self) -> Result<Option<i64>> {
        self.get_setting("active_deck_id")?
            .map(|value| {
                value.parse::<i64>().map_err(|error| {
                    Error::InvalidInput(format!("invalid active_deck_id '{value}': {error}"))
                })
            })
            .transpose()
    }

    pub fn get_active_deck(&self) -> Result<Option<Deck>> {
        self.get_active_deck_id()?
            .map(|deck_id| self.get_deck_by_id(deck_id))
            .transpose()
            .map(Option::flatten)
    }

    pub fn set_active_deck_id(&self, deck_id: Option<i64>) -> Result<()> {
        match deck_id {
            Some(deck_id) => self.set_setting("active_deck_id", &deck_id.to_string())?,
            None => self.delete_setting("active_deck_id")?,
        }
        self.set_current_card_id(None)
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

    pub fn get_deck_by_name(&self, name: &str) -> Result<Option<Deck>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, created_at FROM decks WHERE name = ?1",
            params![name],
            |row| {
                Ok(Deck {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        );
        match result {
            Ok(deck) => Ok(Some(deck)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }

    pub fn get_deck_by_id(&self, id: i64) -> Result<Option<Deck>> {
        let result = self.conn.query_row(
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
        );
        match result {
            Ok(deck) => Ok(Some(deck)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }

    pub fn ensure_deck(&self, name: &str, description: Option<&str>) -> Result<Deck> {
        if let Some(deck) = self.get_deck_by_name(name)? {
            if deck.description.is_none() && description.is_some() {
                self.conn.execute(
                    "UPDATE decks SET description = ?1 WHERE id = ?2",
                    params![description, deck.id],
                )?;
                return self.get_deck_by_name(name)?.ok_or_else(|| {
                    Error::NotFound(format!("deck disappeared after update: {name}"))
                });
            }
            return Ok(deck);
        }
        self.insert_deck(name, description)
    }

    // ── Card ──────────────────────────────────────────────────────────────

    pub fn list_cards_by_deck(&self, deck_name: &str) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.word, c.language, c.meanings, c.pronunciations,
                    c.tags, c.source_name, c.source_license, c.created_at
             FROM cards c
             JOIN decks d ON c.deck_id = d.id
             WHERE d.name = ?1
             ORDER BY c.created_at, c.id",
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
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        rows.into_iter()
            .map(
                |(
                    id,
                    deck_id,
                    word,
                    language,
                    meanings_json,
                    pronunciations_json,
                    tags_json,
                    source_name,
                    source_license,
                    created_at,
                )| {
                    Ok(Card {
                        id,
                        deck_id,
                        word,
                        language,
                        meanings: serde_json::from_str(&meanings_json)?,
                        pronunciations: serde_json::from_str(&pronunciations_json)?,
                        tags: serde_json::from_str(&tags_json)?,
                        source: source_name.map(|name| Source {
                            name,
                            license: source_license,
                        }),
                        created_at,
                    })
                },
            )
            .collect()
    }

    pub fn list_cards_by_deck_paginated(
        &self,
        deck_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.word, c.language, c.meanings, c.pronunciations,
                    c.tags, c.source_name, c.source_license, c.created_at
             FROM cards c
             WHERE c.deck_id = ?1
             ORDER BY c.created_at, c.id
             LIMIT ?2 OFFSET ?3",
        )?;
        let cards = stmt
            .query_map(params![deck_id, limit, offset], card_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(cards)
    }

    pub fn insert_card(
        &self,
        deck_id: i64,
        word: &str,
        meanings: &[Meaning],
        pronunciations: &[Pronunciation],
    ) -> Result<Card> {
        self.insert_card_with_metadata(
            deck_id,
            &ImportCard {
                word: word.to_string(),
                language: "en".to_string(),
                meanings: meanings.to_vec(),
                pronunciations: pronunciations.to_vec(),
                tags: Vec::new(),
                source: None,
            },
        )
    }

    pub fn insert_card_with_metadata(&self, deck_id: i64, card: &ImportCard) -> Result<Card> {
        let meanings_json = serde_json::to_string(&card.meanings)?;
        let pronunciations_json = serde_json::to_string(&card.pronunciations)?;
        let tags_json = serde_json::to_string(&card.tags)?;
        self.conn.execute(
            "INSERT INTO cards
             (deck_id, word, language, meanings, pronunciations, tags, source_name, source_license)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                deck_id,
                card.word,
                card.language,
                meanings_json,
                pronunciations_json,
                tags_json,
                card.source.as_ref().map(|source| source.name.as_str()),
                card.source
                    .as_ref()
                    .and_then(|source| source.license.as_deref())
            ],
        )?;
        let id = self.conn.last_insert_rowid();

        // Insert a default card_state row.
        self.conn.execute(
            "INSERT OR IGNORE INTO card_state (card_id) VALUES (?1)",
            params![id],
        )?;

        self.get_card_by_id(id)?
            .ok_or_else(|| Error::NotFound(format!("card id {id}")))
    }

    pub fn import_cards(
        &self,
        deck_name: &str,
        deck_description: Option<&str>,
        cards: &[ImportCard],
        duplicate_strategy: DuplicateStrategy,
    ) -> Result<ImportSummary> {
        let deck = self.ensure_deck(deck_name, deck_description)?;
        let mut summary = ImportSummary {
            deck_id: deck.id,
            deck_name: deck.name,
            input_count: cards.len(),
            inserted: 0,
            updated: 0,
            skipped: 0,
            merged: 0,
        };

        for import_card in cards {
            let existing = self.find_first_card_by_word(deck.id, &import_card.word)?;
            match (existing, duplicate_strategy) {
                (None, _) | (Some(_), DuplicateStrategy::Keep) => {
                    self.insert_card_with_metadata(deck.id, import_card)?;
                    summary.inserted += 1;
                }
                (Some(_), DuplicateStrategy::Skip) => {
                    summary.skipped += 1;
                }
                (Some(existing), DuplicateStrategy::Overwrite) => {
                    self.update_card_from_import(existing.id, import_card)?;
                    summary.updated += 1;
                }
                (Some(existing), DuplicateStrategy::Merge) => {
                    let merged = merge_import_card(&existing, import_card);
                    self.update_card_from_import(existing.id, &merged)?;
                    summary.merged += 1;
                }
            }
        }

        Ok(summary)
    }

    pub fn get_card_by_id(&self, id: i64) -> Result<Option<Card>> {
        let result = self.conn.query_row(
            "SELECT id, deck_id, word, language, meanings, pronunciations, tags,
                    source_name, source_license, created_at
             FROM cards WHERE id = ?1",
            params![id],
            card_from_row,
        );
        match result {
            Ok(card) => Ok(Some(card)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }

    pub fn get_current_card_id(&self) -> Result<Option<i64>> {
        self.get_setting("current_card_id")?
            .map(|value| {
                value.parse::<i64>().map_err(|error| {
                    Error::InvalidInput(format!("invalid current_card_id '{value}': {error}"))
                })
            })
            .transpose()
    }

    pub fn set_current_card_id(&self, card_id: Option<i64>) -> Result<()> {
        match card_id {
            Some(card_id) => self.set_setting("current_card_id", &card_id.to_string()),
            None => self.delete_setting("current_card_id"),
        }
    }

    pub fn get_current_card(&self) -> Result<Option<Card>> {
        self.get_current_card_id()?
            .map(|card_id| self.get_card_by_id(card_id))
            .transpose()
            .map(Option::flatten)
    }

    pub fn get_current_card_in_deck(&self, deck_id: i64) -> Result<Option<Card>> {
        Ok(self
            .get_current_card()?
            .filter(|card| card.deck_id == deck_id))
    }

    pub fn list_cards_with_state(&self) -> Result<Vec<CardWithState>> {
        self.list_cards_with_state_filtered(None)
    }

    pub fn list_cards_with_state_by_deck(&self, deck_id: i64) -> Result<Vec<CardWithState>> {
        self.list_cards_with_state_filtered(Some(deck_id))
    }

    fn list_cards_with_state_filtered(&self, deck_id: Option<i64>) -> Result<Vec<CardWithState>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.word, c.language, c.meanings, c.pronunciations,
                    c.tags, c.source_name, c.source_license, c.created_at,
                    s.card_id, s.stability, s.difficulty, s.due, s.reps, s.lapses, s.state,
                    (
                        SELECT MAX(reviewed_at)
                        FROM review_log r
                        WHERE r.card_id = c.id
                    ) AS last_reviewed_at
             FROM cards c
             JOIN card_state s ON s.card_id = c.id
             WHERE (?1 IS NULL OR c.deck_id = ?1)
             ORDER BY c.id",
        )?;
        let rows = stmt
            .query_map(params![deck_id], |row| {
                let card = card_from_row(row)?;
                let state_str = row.get::<_, String>(16)?;
                let state = state_str.parse::<ReviewState>().unwrap_or(ReviewState::New);
                Ok(CardWithState {
                    card,
                    state: CardState {
                        card_id: row.get(10)?,
                        stability: row.get(11)?,
                        difficulty: row.get(12)?,
                        due: row.get(13)?,
                        reps: row.get(14)?,
                        lapses: row.get(15)?,
                        state,
                    },
                    last_reviewed_at: row.get(17)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn last_reviewed_at(&self, card_id: i64) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT MAX(reviewed_at) FROM review_log WHERE card_id = ?1",
            params![card_id],
            |row| row.get::<_, Option<String>>(0),
        )?;
        Ok(result)
    }

    pub fn review_log_count(&self, card_id: i64) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM review_log WHERE card_id = ?1",
            params![card_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn progress_counts(&self, daily_new_limit: i64) -> Result<ProgressCounts> {
        self.progress_counts_filtered(None, daily_new_limit)
    }

    pub fn progress_counts_by_deck(
        &self,
        deck_id: i64,
        daily_new_limit: i64,
    ) -> Result<ProgressCounts> {
        self.progress_counts_filtered(Some(deck_id), daily_new_limit)
    }

    fn progress_counts_filtered(
        &self,
        deck_id: Option<i64>,
        daily_new_limit: i64,
    ) -> Result<ProgressCounts> {
        let due_count = self.conn.query_row(
            "SELECT COUNT(*)
             FROM card_state s
             JOIN cards c ON c.id = s.card_id
             WHERE (?1 IS NULL OR c.deck_id = ?1)
               AND s.reps > 0
               AND s.due <= datetime('now')",
            params![deck_id],
            |row| row.get(0),
        )?;
        let new_count = self.conn.query_row(
            "SELECT COUNT(*)
             FROM card_state s
             JOIN cards c ON c.id = s.card_id
             WHERE (?1 IS NULL OR c.deck_id = ?1)
               AND s.reps = 0",
            params![deck_id],
            |row| row.get::<_, i64>(0),
        )?;
        let reviewed_today = self.conn.query_row(
            "SELECT COUNT(*)
             FROM review_log r
             JOIN cards c ON c.id = r.card_id
             WHERE (?1 IS NULL OR c.deck_id = ?1)
               AND date(r.reviewed_at) = date('now')",
            params![deck_id],
            |row| row.get(0),
        )?;
        Ok(ProgressCounts {
            due_count,
            new_remaining: daily_new_limit.min(new_count),
            reviewed_today,
        })
    }

    pub fn card_count_by_deck(&self, deck_id: i64) -> Result<i64> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1",
            params![deck_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn review_stats_by_deck_and_day_range(
        &self,
        deck_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<DailyReviewBucket>> {
        let mut stmt = self.conn.prepare(
            "SELECT date(r.reviewed_at) AS day,
                    COUNT(*) AS reviews,
                    SUM(CASE WHEN r.rating = 1 THEN 1 ELSE 0 END) AS again,
                    SUM(CASE WHEN r.rating = 2 THEN 1 ELSE 0 END) AS hard,
                    SUM(CASE WHEN r.rating = 3 THEN 1 ELSE 0 END) AS good,
                    SUM(CASE WHEN r.rating = 4 THEN 1 ELSE 0 END) AS easy
             FROM review_log r
             JOIN cards c ON c.id = r.card_id
             WHERE c.deck_id = ?1
               AND date(r.reviewed_at) >= ?2
               AND date(r.reviewed_at) <= ?3
             GROUP BY day
             ORDER BY day",
        )?;
        let rows = stmt
            .query_map(params![deck_id, start_date, end_date], |row| {
                Ok(DailyReviewBucket {
                    date: row.get(0)?,
                    stats: DailyReviewStats {
                        reviews: row.get(1)?,
                        again: row.get(2)?,
                        hard: row.get(3)?,
                        good: row.get(4)?,
                        easy: row.get(5)?,
                    },
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn record_review(&self, review: &ScheduledReview) -> Result<()> {
        self.conn.execute(
            "UPDATE card_state
             SET stability = ?1,
                 difficulty = ?2,
                 due = ?3,
                 reps = reps + 1,
                 lapses = lapses + ?4,
                 state = ?5
             WHERE card_id = ?6",
            params![
                review.stability,
                review.difficulty,
                review.due,
                i64::from(matches!(review.rating, crate::card::Rating::Again)),
                review.state.to_string(),
                review.card_id
            ],
        )?;
        self.conn.execute(
            "INSERT INTO review_log
             (card_id, rating, reviewed_at, elapsed_days, scheduled_days)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                review.card_id,
                review.rating.as_i64(),
                review.reviewed_at,
                review.elapsed_days,
                review.scheduled_days
            ],
        )?;
        Ok(())
    }

    fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value)
             VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn delete_setting(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    fn find_first_card_by_word(&self, deck_id: i64, word: &str) -> Result<Option<Card>> {
        let result = self.conn.query_row(
            "SELECT id, deck_id, word, language, meanings, pronunciations, tags,
                    source_name, source_license, created_at
             FROM cards
             WHERE deck_id = ?1 AND word = ?2
             ORDER BY id
             LIMIT 1",
            params![deck_id, word],
            card_from_row,
        );
        match result {
            Ok(card) => Ok(Some(card)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Db(e)),
        }
    }

    fn update_card_from_import(&self, id: i64, card: &ImportCard) -> Result<()> {
        let meanings_json = serde_json::to_string(&card.meanings)?;
        let pronunciations_json = serde_json::to_string(&card.pronunciations)?;
        let tags_json = serde_json::to_string(&card.tags)?;
        self.conn.execute(
            "UPDATE cards
             SET word = ?1,
                 language = ?2,
                 meanings = ?3,
                 pronunciations = ?4,
                 tags = ?5,
                 source_name = ?6,
                 source_license = ?7
             WHERE id = ?8",
            params![
                card.word,
                card.language,
                meanings_json,
                pronunciations_json,
                tags_json,
                card.source.as_ref().map(|source| source.name.as_str()),
                card.source
                    .as_ref()
                    .and_then(|source| source.license.as_deref()),
                id
            ],
        )?;
        Ok(())
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
                let state = state_str.parse::<ReviewState>().unwrap_or(ReviewState::New);
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

fn card_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    let meanings_json = row.get::<_, String>(4)?;
    let pronunciations_json = row.get::<_, String>(5)?;
    let tags_json = row.get::<_, String>(6)?;
    let source_name = row.get::<_, Option<String>>(7)?;
    Ok(Card {
        id: row.get(0)?,
        deck_id: row.get(1)?,
        word: row.get(2)?,
        language: row.get(3)?,
        meanings: serde_json::from_str(&meanings_json).map_err(json_to_sql_error)?,
        pronunciations: serde_json::from_str(&pronunciations_json).map_err(json_to_sql_error)?,
        tags: serde_json::from_str(&tags_json).map_err(json_to_sql_error)?,
        source: source_name.map(|name| Source {
            name,
            license: row.get::<_, Option<String>>(8).ok().flatten(),
        }),
        created_at: row.get(9)?,
    })
}

fn json_to_sql_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn merge_import_card(existing: &Card, incoming: &ImportCard) -> ImportCard {
    let mut meanings = existing.meanings.clone();
    for meaning in &incoming.meanings {
        if !meanings.iter().any(|item| {
            item.part_of_speech == meaning.part_of_speech && item.definition == meaning.definition
        }) {
            meanings.push(meaning.clone());
        }
    }

    let mut pronunciations = existing.pronunciations.clone();
    for pronunciation in &incoming.pronunciations {
        if !pronunciations
            .iter()
            .any(|item| item.notation == pronunciation.notation)
        {
            pronunciations.push(pronunciation.clone());
        }
    }

    let mut tags = existing.tags.clone();
    for tag in &incoming.tags {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }

    ImportCard {
        word: existing.word.clone(),
        language: if incoming.language.is_empty() {
            existing.language.clone()
        } else {
            incoming.language.clone()
        },
        meanings,
        pronunciations,
        tags,
        source: incoming.source.clone().or_else(|| existing.source.clone()),
    }
}

fn ensure_card_metadata_columns(conn: &Connection) -> Result<()> {
    let columns = table_columns(conn, "cards")?;
    let missing = [
        (
            "language",
            "ALTER TABLE cards ADD COLUMN language TEXT NOT NULL DEFAULT 'en'",
        ),
        (
            "tags",
            "ALTER TABLE cards ADD COLUMN tags TEXT NOT NULL DEFAULT '[]'",
        ),
        (
            "source_name",
            "ALTER TABLE cards ADD COLUMN source_name TEXT",
        ),
        (
            "source_license",
            "ALTER TABLE cards ADD COLUMN source_license TEXT",
        ),
    ];

    for (name, statement) in missing {
        if !columns.iter().any(|column| column == name) {
            conn.execute(statement, [])?;
        }
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cards_deck_word ON cards(deck_id, word)",
        [],
    )?;
    Ok(())
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(columns)
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
        let deck = storage
            .insert_deck("cet4", Some("CET-4 vocabulary"))
            .unwrap();
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
        let card = storage.insert_card(deck.id, "hello", &[], &[]).unwrap();
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
    fn test_list_cards_by_deck_paginated() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", None).unwrap();
        storage.insert_card(deck.id, "first", &[], &[]).unwrap();
        storage.insert_card(deck.id, "second", &[], &[]).unwrap();
        storage.insert_card(deck.id, "third", &[], &[]).unwrap();

        let first_page = storage.list_cards_by_deck_paginated(deck.id, 2, 0).unwrap();
        let second_page = storage.list_cards_by_deck_paginated(deck.id, 2, 2).unwrap();

        assert_eq!(storage.card_count_by_deck(deck.id).unwrap(), 3);
        assert_eq!(
            first_page
                .iter()
                .map(|card| card.word.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second"]
        );
        assert_eq!(
            second_page
                .iter()
                .map(|card| card.word.as_str())
                .collect::<Vec<_>>(),
            vec!["third"]
        );
    }

    #[test]
    fn test_active_deck_setting_clears_current_card() {
        let storage = open_temp();
        let deck = storage.insert_deck("cet4", None).unwrap();
        let card = storage.insert_card(deck.id, "cancel", &[], &[]).unwrap();

        storage.set_current_card_id(Some(card.id)).unwrap();
        storage.set_active_deck_id(Some(deck.id)).unwrap();

        assert_eq!(storage.get_active_deck_id().unwrap(), Some(deck.id));
        assert_eq!(storage.get_active_deck().unwrap().unwrap().name, "cet4");
        assert_eq!(storage.get_current_card_id().unwrap(), None);
    }

    #[test]
    fn test_progress_counts_are_scoped_by_deck() {
        let storage = open_temp();
        let first_deck = storage.insert_deck("first", None).unwrap();
        let second_deck = storage.insert_deck("second", None).unwrap();
        storage
            .insert_card(first_deck.id, "same", &[], &[])
            .unwrap();
        storage
            .insert_card(second_deck.id, "same", &[], &[])
            .unwrap();

        let first_progress = storage.progress_counts_by_deck(first_deck.id, 20).unwrap();
        let second_progress = storage.progress_counts_by_deck(second_deck.id, 20).unwrap();

        assert_eq!(first_progress.new_remaining, 1);
        assert_eq!(second_progress.new_remaining, 1);
    }

    #[test]
    fn test_default_path_is_some() {
        // Just ensure it doesn't error on this platform.
        let path = Storage::default_path().unwrap();
        assert!(path.to_str().unwrap().contains("fishword"));
    }

    #[test]
    fn test_open_upgrades_m1_cards_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep().join("old.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE decks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE cards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
                word TEXT NOT NULL,
                meanings TEXT NOT NULL DEFAULT '[]',
                pronunciations TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .unwrap();
        drop(conn);

        let storage = Storage::open(&path).unwrap();
        let deck = storage.insert_deck("legacy", None).unwrap();
        let card = storage.insert_card(deck.id, "hello", &[], &[]).unwrap();
        assert_eq!(card.language, "en");
        assert!(card.tags.is_empty());
    }
}
