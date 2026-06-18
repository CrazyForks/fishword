# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root — covers the shared domain: CLI protocol, core data model, FSRS scheduling, and deck/card vocabulary.
- **`packages/<extension>/CONTEXT.md`** — if the task touches a specific extension, also read its context file (if it exists).
- **`docs/adr/`** at the repo root — system-wide architectural decisions.
- **`packages/<extension>/docs/adr/`** — extension-scoped decisions (if they exist).

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill creates them lazily when terms or decisions actually get resolved.

## File structure

This repo uses a **single-core + multi-extension** layout:

```
/
├── CONTEXT.md                    ← global domain: CLI protocol, core data model, FSRS scheduling
├── docs/adr/                     ← system-wide architectural decisions
├── crates/
│   ├── fishword-core/
│   └── fishword-cli/
└── packages/
    └── pi-extension/
        ├── CONTEXT.md            ← Pi extension domain language (optional, skip if absent)
        └── docs/adr/             ← Pi extension architectural decisions (optional)
```

Future extensions follow the same convention: `packages/<name>/CONTEXT.md` and `packages/<name>/docs/adr/`.

## Reading order

1. Always read root `CONTEXT.md` first.
2. If the task involves a specific extension, also read `packages/<extension>/CONTEXT.md`.
3. Read relevant ADRs from `docs/adr/` and `packages/<extension>/docs/adr/`.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders) — but worth reopening because…_
