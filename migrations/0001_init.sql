PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS decks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    description TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS cards (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id        INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    word           TEXT    NOT NULL,
    language       TEXT    NOT NULL DEFAULT 'en',
    meanings       TEXT    NOT NULL DEFAULT '[]',
    pronunciations TEXT    NOT NULL DEFAULT '[]',
    tags           TEXT    NOT NULL DEFAULT '[]',
    source_name    TEXT,
    source_license TEXT,
    created_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS card_state (
    card_id    INTEGER PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
    stability  REAL    NOT NULL DEFAULT 0.0,
    difficulty REAL    NOT NULL DEFAULT 0.0,
    due        TEXT    NOT NULL DEFAULT (datetime('now')),
    reps       INTEGER NOT NULL DEFAULT 0,
    lapses     INTEGER NOT NULL DEFAULT 0,
    state      TEXT    NOT NULL DEFAULT 'new'
        CHECK (state IN ('new', 'learning', 'review', 'relearning'))
);

CREATE TABLE IF NOT EXISTS review_log (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id        INTEGER NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    rating         INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
    reviewed_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    elapsed_days   INTEGER NOT NULL DEFAULT 0,
    scheduled_days INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_cards_deck_id   ON cards(deck_id);
CREATE INDEX IF NOT EXISTS idx_cards_deck_word ON cards(deck_id, word);
CREATE INDEX IF NOT EXISTS idx_card_state_due  ON card_state(due);
CREATE INDEX IF NOT EXISTS idx_review_log_card ON review_log(card_id);
