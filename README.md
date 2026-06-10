# Fishword

Fishword 是一个本地词汇学习项目。当前核心是 Rust CLI，负责词库导入、SQLite 存储、FSRS 复习调度和稳定 JSON 协议。

## 文档

- 当前开发流程：[docs/development.md](./docs/development.md)
- 用户使用说明草案：[docs/usage.md](./docs/usage.md)


## 开发环境

本项目使用 pnpm 管理 JS/Pi workspace

安装 workspace：

```bash
pnpm install
```

## CLI 开发流程

日常开发 Rust CLI 时，先构建 debug 版本：

```bash
pnpm dev:cli
```

这会生成：

```text
target/debug/fishword
```

然后跑 CLI 冒烟测试：

```bash
pnpm smoke:cli
```

`smoke:cli` 会使用临时 `HOME`，不会污染你的真实数据库。当前覆盖：

```text
fishword init
fishword import qwerty
fishword deck current
fishword deck use <deck>
fishword current --json
fishword rate good --json
```

也可以直接用 Cargo 手动调试：

```bash
cargo run -p fishword-cli -- init
cargo run -p fishword-cli -- current --json
cargo run -p fishword-cli -- rate good --json
```

手动测试时建议使用隔离 `HOME`：

```bash
HOME=/private/tmp/fishword-dev cargo run -p fishword-cli -- init
HOME=/private/tmp/fishword-dev cargo run -p fishword-cli -- import qwerty assets/dicts/qwerty-learner/dicts/CET4_T.json --deck cet4 --name "CET-4"
HOME=/private/tmp/fishword-dev cargo run -p fishword-cli -- current --json
```

## 测试与检查

Rust 测试：

```bash
pnpm test:rust
```

完整本地检查：

```bash
pnpm check
```

`pnpm check` 当前会执行：

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
node scripts/smoke-cli.mjs
```

如果只想单独跑某一项：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
node scripts/smoke-cli.mjs
```

## 本机部署测试

如果想像普通用户一样在任意目录执行 `fishword`，可以把本地 `@fishword/cli` wrapper link 到全局。

先构建 CLI：

```bash
pnpm dev:cli
```

再全局 link：

```bash
cd packages/cli
pnpm link --global
cd ../..
```

验证：

```bash
fishword --version
fishword --help
```

用隔离数据库做完整手动测试：

```bash
HOME=/private/tmp/fishword-dev fishword init
HOME=/private/tmp/fishword-dev fishword import qwerty assets/dicts/qwerty-learner/dicts/CET4_T.json --deck cet4 --name "CET-4"
HOME=/private/tmp/fishword-dev fishword deck use cet4
HOME=/private/tmp/fishword-dev fishword current --json
HOME=/private/tmp/fishword-dev fishword rate good --json
```

这个全局 `fishword` 实际是 JS wrapper。开发模式下它会解析到当前仓库的：

```text
target/debug/fishword
```

所以每次修改 Rust 后，重新执行：

```bash
pnpm dev:cli
```

全局命令就会使用最新的 debug binary。

## 默认词库

内置词库来源及许可证说明见 [assets/dicts/qwerty-learner/SOURCE.md](./assets/dicts/qwerty-learner/SOURCE.md)。
