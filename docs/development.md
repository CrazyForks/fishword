# Development

## CLI Loop

Use Cargo directly while working on Rust logic:

```bash
cargo run -p fishword-cli -- current --json
cargo run -p fishword-cli -- next --json
cargo run -p fishword-cli -- rate good --json
```

Use an isolated `HOME` for manual testing:

```bash
HOME=/private/tmp/fishword-dev cargo run -p fishword-cli -- init
HOME=/private/tmp/fishword-dev cargo run -p fishword-cli -- current --json
```

## pnpm Workspace

M5 adds pnpm as the outer workspace for JavaScript/Pi packages. Cargo still owns
Rust dependencies and compilation.

Install pnpm before using workspace commands:

```bash
corepack enable
corepack prepare pnpm@9.15.4 --activate
```

Expected local commands:

```bash
pnpm dev:cli
pnpm smoke:cli
pnpm test:rust
pnpm check
```

`pnpm dev:cli` builds `target/debug/fishword`. `@fishword/cli` resolves that
debug binary first, so JS adapters can use the same import path in development
and production:

```js
import { fishwordPath } from "@fishword/cli";
```

## CLI Wrapper

`packages/cli` provides:

```text
@fishword/cli
  exports fishwordPath
  bin fishword
```

Resolution order:

```text
FISHWORD_CLI_PATH
target/debug/fishword
@fishword/cli-<platform>/bin/fishword
```

The platform packages are intentionally thin. They only carry the compiled Rust
binary for one OS/CPU pair.

## Smoke Test

The smoke test runs against an isolated temporary `HOME`:

```bash
pnpm dev:cli
pnpm smoke:cli
```

It verifies:

```text
fishword init
fishword import qwerty
fishword current --json
fishword next --json
fishword rate good --json
```

## Pi Extension Loop

After M6 adds the extension package, local development should use:

```bash
pnpm dev:cli
pi -e ./packages/pi-extension
```

The extension should import `fishwordPath` from `@fishword/cli`, call the Rust
CLI through `execFile`, and parse only JSON protocol output.
