# ADR-0002: CLI is the stable integration boundary; frontends use JSON protocol only

## Status

Accepted

## Context

Fishword has multiple frontend consumers: the Pi extension, npm wrapper packages, and future terminal/editor integrations. These consumers need a stable contract with the Rust core.

## Decision

The CLI is the single public boundary of the system. All frontend integrations must use JSON protocol commands (`--json` flag) rather than parsing human-readable text output.

Human-readable output (`--format plain`, `--format compact`, `--format status`) is available for manual testing only and carries no stability guarantee.

## Consequences

- Adding or changing human-readable output never breaks integrations.
- The Pi extension and any future extension must not parse plain-text CLI output.
- New CLI commands that produce structured data must implement a `--json` response using the protocol DTOs in `fishword-core::protocol`.
