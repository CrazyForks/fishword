# Fishword Project Guide

## Project Overview

Fishword is a local vocabulary learning project built around a Rust CLI. The CLI
is intended to become the stable integration layer for multiple frontends,
including a Pi extension, terminal UI, and future agent/editor integrations.

The project currently supports:

- SQLite-backed decks, cards, card state, and review logs
- Importing Qwerty Learner JSON, CSV, JSONL, and Anki TSV
- Bundled default dictionaries from Qwerty Learner
- FSRS-based review scheduling
- Card selection through `current`, `next`, and `rate`
- Stable JSON protocol output for frontend integrations

## CLI Role

The CLI is the public boundary of the system. Frontends should prefer JSON
protocol commands instead of parsing human-readable text.

Core commands:

```bash
fishword init
fishword import qwerty <file> --deck <deck> --name <name>
fishword current --json
fishword next --json
fishword rate good --json
```

Human-readable output remains available for manual testing:

```bash
fishword current --format plain
fishword current --format compact
fishword current --format status
```

## Broad Directory Layout

```text
.
├── adapters/
│   └── pi-fishword/
│       └── examples/
│           └── probe.ts
├── assets/
│   └── dicts/
│       └── qwerty-learner/
│           ├── SOURCE.md
│           ├── dicts/
│           └── upstream/
├── crates/
│   ├── fishword-cli/
│   │   └── src/main.rs
│   └── fishword-core/
│       ├── fixtures/
│       └── src/
│           ├── card/
│           ├── deck/
│           ├── importer/
│           ├── protocol/
│           ├── scheduler/
│           ├── selector/
│           └── storage/
├── docs/
│   └── milestones/
├── migrations/
│   └── 0001_init.sql
├── schemas/
│   ├── deck.v1.schema.json
│   └── protocol.v1.schema.json
├── Cargo.toml
└── README.md
```

## Core Crates

### `crates/fishword-core`

Contains the reusable domain logic:

- `card`: card, meaning, pronunciation, review state, rating, and source models
- `deck`: deck model
- `storage`: SQLite persistence, migrations, current-card state, review logs
- `importer`: Qwerty JSON, CSV, JSONL, and Anki TSV importers
- `scheduler`: FSRS review scheduling
- `selector`: current/next card selection policy
- `protocol`: stable JSON DTOs for frontend consumers

### `crates/fishword-cli`

Contains the command-line interface. It should stay thin and delegate domain
work to `fishword-core`.

## Default Dictionaries

Bundled dictionaries live under:

```text
assets/dicts/qwerty-learner/dicts/
```

They are copied from Qwerty Learner and tracked through Git LFS. Keep the
source notice and GPL-3.0 license files under:

```text
assets/dicts/qwerty-learner/SOURCE.md
assets/dicts/qwerty-learner/upstream/LICENSE
```

## Data Storage

The default database path is platform-specific. On macOS it is:

```text
~/Library/Application Support/fishword/fishword.db
```

For isolated manual tests, override `HOME`:

```bash
HOME=/private/tmp/fishword-test ./target/debug/fishword init
```

## Development Notes

- Use Rust workspace commands from the repository root.
- Keep frontend-facing integrations on the JSON protocol.
- Do not parse human-readable CLI output in adapters.
- `next` and `current` must not write review logs.
- Only explicit `rate again|hard|good|easy` writes `review_log` and updates
  `card_state`.
- Keep default dictionary JSON files tracked by Git LFS.

Useful verification commands:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```
