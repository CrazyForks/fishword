# @vocabber/cli

JavaScript wrapper for the Vocabber Rust CLI.

```js
import { vocabbarPath } from "@vocabber/cli";
```

The wrapper resolves binaries in this order:

```text
VOCABBAR_CLI_PATH
target/debug/vocabbar
@vocabber/cli-<platform>/bin/vocabbar
```

It also exposes a `vocabbar` npm binary.
