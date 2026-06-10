# Vocabber Project Guide

## Project Overview

Vocabber is a local vocabulary learning project built around a Rust CLI. The CLI
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
vocabbar init
vocabbar import qwerty <file> --deck <deck> --name <name>
vocabbar current --json
vocabbar next --json
vocabbar rate good --json
```

Human-readable output remains available for manual testing:

```bash
vocabbar current --format plain
vocabbar current --format compact
vocabbar current --format status
```

## Broad Directory Layout

```text
.
├── adapters/
│   └── pi-vocabbar/
│       └── examples/
│           └── probe.ts
├── assets/
│   └── dicts/
│       └── qwerty-learner/
│           ├── SOURCE.md
│           ├── dicts/
│           └── upstream/
├── crates/
│   ├── vocabbar-cli/
│   │   └── src/main.rs
│   └── vocabbar-core/
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

### `crates/vocabbar-core`

Contains the reusable domain logic:

- `card`: card, meaning, pronunciation, review state, rating, and source models
- `deck`: deck model
- `storage`: SQLite persistence, migrations, current-card state, review logs
- `importer`: Qwerty JSON, CSV, JSONL, and Anki TSV importers
- `scheduler`: FSRS review scheduling
- `selector`: current/next card selection policy
- `protocol`: stable JSON DTOs for frontend consumers

### `crates/vocabbar-cli`

Contains the command-line interface. It should stay thin and delegate domain
work to `vocabbar-core`.

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
~/Library/Application Support/vocabbar/vocabbar.db
```

For isolated manual tests, override `HOME`:

```bash
HOME=/private/tmp/vocabbar-test ./target/debug/vocabbar init
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
