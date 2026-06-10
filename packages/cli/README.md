# @fishword/cli

JavaScript wrapper for the Fishword Rust CLI.

```js
import { fishwordPath } from "@fishword/cli";
```

The wrapper resolves binaries in this order:

```text
FISHWORD_CLI_PATH
target/debug/fishword
@fishword/cli-<platform>/bin/fishword
```

It also exposes a `fishword` npm binary.
