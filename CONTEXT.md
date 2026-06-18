# Fishword Domain Context

## Core principle

Each deck runs FSRS independently. Review scheduling, card selection, and progress tracking are always scoped to a single deck. There is no cross-deck scheduling or selection.

## Glossary

**Deck** — an independent vocabulary set with its own FSRS state. A user may have multiple decks (e.g. CET-4, CET-6, TOEFL); their scheduling never interacts.

**Card** — a vocabulary item belonging to exactly one deck. Has its own FSRS state (`card_state` row).

**Review** — a single rating event (`again | hard | good | easy`) that advances a card's FSRS state and updates the current-card pointer within its deck.

**Current card** — the card a user is actively studying, scoped to a deck. Stored as `current_card_id` in the `settings` table; always interpreted in the context of a specific deck.

**Selection** — the process of choosing which card to show next within a deck, based on FSRS due dates, retrievability, and the daily new-card limit.

**Daily new limit** — the maximum number of new (never-reviewed) cards introduced per deck per day. Default: 20.

**Catalog** — a remote index of pre-built decks downloadable via `fishword catalog fetch`. Separate from local decks; fetched decks become independent local decks after import.
