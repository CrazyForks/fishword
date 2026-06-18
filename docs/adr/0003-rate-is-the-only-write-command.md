# ADR-0003: Only `rate` writes review_log and card_state

## Status

Accepted

## Context

Several commands read the current card or display statistics. It would be tempting to let `current` or `status` lazily initialise card state or log implicit "views" for analytics.

## Decision

`current` and `status` are strictly read-only. Only an explicit `rate again|hard|good|easy` writes a `review_log` entry and updates `card_state`.

## Consequences

- Review counts and FSRS state reflect only deliberate user ratings, never side effects of navigation.
- `current` and `status` can be called freely without altering scheduling state.
- Any future command that reads card data must not write `review_log` or `card_state` as a side effect.
