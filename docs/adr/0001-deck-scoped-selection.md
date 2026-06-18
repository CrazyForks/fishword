# ADR-0001: Card selection is always deck-scoped

## Status

Accepted

## Context

The Selector module historically exposed two pairs of methods:

- `select_current` / `select_next` — global, queries cards across all decks
- `select_current_in_deck` / `select_next_in_deck` — scoped to one deck

The global variants were a legacy artifact, not an intentional design. The core product principle is that each deck runs FSRS independently; there is no cross-deck scheduling or selection.

## Decision

Card selection is always scoped to a single deck. The global (cross-deck) variants of `select_current` and `select_next` are to be removed. All callers must pass a `deck_id`.

## Consequences

- The Selector interface shrinks: four methods collapse to two, unified by `deck_id`.
- The `Selector` struct and its mutable `daily_new_limit` field can be removed; `deck_id` becomes a plain parameter.
- Future architecture reviews must not re-introduce cross-deck selection — it contradicts the per-deck FSRS isolation principle.
- CLI commands that previously relied on global selection must resolve a deck before calling the selector.
