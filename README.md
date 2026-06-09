# Vocabber

Vocabber is a local vocabulary CLI and Pi extension project. The current
implementation includes SQLite-backed decks/cards and importers for Qwerty
Learner JSON, CSV, JSONL, and Anki TSV.

## Default Dictionaries

This repository bundles default dictionaries from
[Qwerty Learner](https://github.com/RealKai42/qwerty-learner), a typing-based
vocabulary learning project by RealKai42 and contributors.

- Local path: `assets/dicts/qwerty-learner/dicts/`
- Upstream repository: `https://github.com/RealKai42/qwerty-learner`
- Upstream source directory: `public/dicts/`
- Imported upstream commit: `2498f753aaf955645f466664d3972c2c7d29dd55`
- Dictionary count: 380 JSON files
- Upstream license: GPL-3.0
- Bundled license copy: `assets/dicts/qwerty-learner/upstream/LICENSE`

The bundled dictionaries are redistributed under the upstream GPL-3.0 license.
If you distribute Vocabber with these dictionaries, comply with GPL-3.0 and keep
the attribution and license notice intact.

Common default imports:

```bash
cargo run -p vocabbar-cli -- import qwerty assets/dicts/qwerty-learner/dicts/CET4_T.json --deck cet4 --name "CET-4"
cargo run -p vocabbar-cli -- import qwerty assets/dicts/qwerty-learner/dicts/CET6_T.json --deck cet6 --name "CET-6"
cargo run -p vocabbar-cli -- import qwerty assets/dicts/qwerty-learner/dicts/TOEFL_3_T.json --deck toefl --name "TOEFL"
```

After importing:

```bash
cargo run -p vocabbar-cli -- deck list
cargo run -p vocabbar-cli -- card list --deck cet4
```

## Import Formats

See `docs/importers.md` for supported import formats and duplicate strategies.
