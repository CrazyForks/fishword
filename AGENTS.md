# Fishword Project Guide

## Project Overview

Fishword is a local vocabulary learning project centered on a Rust CLI. The CLI
is the stable integration boundary for the Pi extension, npm packages, and
future terminal/editor integrations.

The project currently supports:

- SQLite-backed decks, cards, card state, settings, and review logs
- Importing fishword.deck.v1 JSONL (the only supported import format; other sources are converted to it offline, e.g. via `scripts/convert-qwerty-decks.mjs`)
- kajweb/dict JSONL dictionaries converted into Fishword's deck.v1 JSONL format
- FSRS-based review scheduling
- Card selection through `current`; `rate` records a review and returns the next card in JSON output
- Stable JSON protocol output for frontend integrations
- npm-distributed CLI wrapper and Pi extension packages
- Online catalog of pre-built decks hosted on GitHub Pages, downloadable via `fishword catalog`

## CLI Role

The CLI is the public boundary of the system. Frontends should prefer JSON
protocol commands instead of parsing human-readable text.

Common command flow:

```bash
fishword init
fishword deck create CET-4 --description "大学英语四级" --json
fishword import jsonl assets/dicts/kajweb/cet4.jsonl --deck-id <numeric-deck-id>
fishword current --json
fishword rate good --json
```

Or use the catalog to download a pre-built deck in one step:

```bash
fishword catalog list
fishword catalog fetch kajweb:cet4
fishword catalog fetch kajweb:toefl --duplicates merge --json
```

Important CLI details:

- `import` currently takes a numeric local deck id via `--deck-id`; create or list decks first.
- `import jsonl <path> --create-deck <name>` creates a new local deck and imports into it.
- `catalog fetch` takes a catalog id such as `kajweb:cet4`, not a local numeric deck id.
- `catalog fetch` creates a new deck automatically (or merges into an existing one with the same name).
- There is no standalone `next` command in the current CLI. Use `current` to select/show the current card, and `rate again|hard|good|easy --json` to record a review and receive the next card.
- Set `FISHWORD_CATALOG_URL` to override the catalog endpoint (useful for offline testing or self-hosted mirrors).
- Human-readable output remains available for manual testing, for example:

```bash
fishword current --format plain
fishword current --format compact
fishword current --format status
fishword status --format statusline
```

## Broad Directory Layout

```text
.
├── assets/
│   └── dicts/
│       ├── kajweb/
│       │   ├── README.md
│       │   ├── cet4.jsonl
│       │   ├── cet6.jsonl
│       │   └── ...
│       └── qwerty-learner/
│           ├── SOURCE.md
│           ├── dicts/
│           └── upstream/
├── crates/
│   ├── fishword-cli/
│   └── fishword-core/
├── docs/
├── migrations/
│   └── 0001_init.sql
├── packages/
│   ├── cli/
│   ├── cli-darwin-arm64/
│   ├── cli-darwin-x64/
│   ├── cli-linux-arm64/
│   ├── cli-linux-x64/
│   ├── cli-win32-x64/
│   └── pi-extension/
├── schemas/
├── scripts/
│   ├── convert-qwerty-decks.mjs
│   ├── kajweb_to_jsonl.py
│   ├── prepare-pi-extension-assets.mjs
│   └── smoke-cli.mjs
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
└── README.md
```

## Core Crates

### `crates/fishword-core`

Contains reusable domain logic:

- `card`: card, meaning, pronunciation, review state, rating, and source models
- `deck`: deck model
- `storage`: SQLite persistence, migrations, settings, current-card state, review logs
- `importer`: deck.v1 JSONL importer (the only supported runtime import format)
- `scheduler`: FSRS review scheduling
- `selector`: current-card and next-card selection policy
- `protocol`: stable JSON DTOs for frontend consumers

### `crates/fishword-cli`

Contains the command-line interface. Keep it thin and delegate domain work to
`fishword-core`.

## npm Packages and Pi Extension

The JavaScript workspace is managed by pnpm under `packages/`.

- `@fishword/cli` is the JavaScript wrapper and exposes the `fishword` npm binary.
- `@fishword/cli-*` packages contain platform-specific Rust binaries.
- `@fishword/pi-extension` is the Pi extension package.

The release workflow builds Rust binaries, publishes platform CLI packages,
publishes `@fishword/cli`, builds the Pi extension, then publishes
`@fishword/pi-extension`.

The Pi extension seeds three default decks on session start:

- `CET-4`
- `CET-6`
- `TOEFL`

The seed logic lives in:

```text
packages/pi-extension/src/defaultDecks.ts
```

It is intentionally driven from the extension, not Rust `init`: the extension
knows where its npm package assets are, while the Rust CLI only receives local
file paths and imports them.

The Pi extension build copies the three default kajweb JSONL files from
`assets/dicts/kajweb/` into package-local assets:

```text
packages/pi-extension/assets/dicts/kajweb/
```

That generated `packages/pi-extension/assets/` directory is ignored by Git but
included in the npm tarball through `packages/pi-extension/package.json`.

## Dictionaries

### kajweb/dict

Converted kajweb dictionaries live under:

```text
assets/dicts/kajweb/
```

They are Fishword deck.v1 JSONL files. The conversion script is:

```bash
uv run scripts/kajweb_to_jsonl.py --book CET4 -o assets/dicts/kajweb/cet4.jsonl
```

When working with Python scripts, use `uv run` as required by this repository.

### Qwerty Learner

Qwerty Learner source dictionaries still live under:

```text
assets/dicts/qwerty-learner/dicts/
```

Keep the source notice and upstream license files under:

```text
assets/dicts/qwerty-learner/SOURCE.md
assets/dicts/qwerty-learner/upstream/LICENSE
```

### Online Catalog

Selected dictionaries are published as fishword.deck.v1 JSONL files to GitHub
Pages and indexed by a `catalog.json` manifest. The build is driven by:

```text
scripts/convert-qwerty-decks.mjs
```

Output goes to `dist/catalog/` (git-ignored). The workflow
`.github/workflows/publish-catalog.yml` deploys this directory to the `gh-pages`
branch under `catalog/` whenever dictionary sources or the script change on main.

The catalog endpoint used by the CLI is:

```
https://chenggou1.github.io/fishword/catalog/catalog.json
```

To regenerate the catalog locally:

```bash
node scripts/convert-qwerty-decks.mjs
```

### Git LFS

Dictionary data files are tracked by Git LFS:

- `assets/dicts/qwerty-learner/dicts/*.json`
- `assets/dicts/kajweb/*.jsonl`

After changing LFS patterns for already tracked files, run:

```bash
git add --renormalize <path>
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
- Use pnpm workspace commands for JavaScript packages.
- Keep frontend-facing integrations on the JSON protocol.
- Do not parse human-readable CLI output in the Pi extension or other integrations.
- `current` and `status` must not write review logs.
- Only explicit `rate again|hard|good|easy` writes `review_log` and updates `card_state`.
- Do not make Rust `init` aware of npm package paths; package-local asset lookup belongs in the Pi extension.
- Keep dictionary data tracked by Git LFS.

## Commit Message Format

When creating Git commits, use the exact `type: message` format.

Examples:

```text
refactor: refine catalog identifiers
fix: handle empty import files
docs: update catalog examples
test: cover import target arguments
```

Use a lowercase type such as `feat`, `fix`, `refactor`, `docs`, `test`,
`chore`, or `ci`, followed by a colon, one space, and a concise imperative
message. Do not use `type!:` / `feat!:` syntax unless the user explicitly asks
for it.

Useful verification commands:

```bash
pnpm check
pnpm format:check
pnpm lint
pnpm test
pnpm check:pi
pnpm smoke:rust
```
