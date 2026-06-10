# 使用说明

本文档面向安装后的 Vocabber 用户，记录稳定的 CLI 使用方式和计划中的安装入口。当前项目仍在开发中，正式 npm 包、平台二进制下载和 Pi extension 发布尚未完成。

## 安装方式

### npm 安装（计划）

正式发布后，推荐通过 npm 安装 CLI：

```bash
npm install -g @vocabber/cli
```

安装后验证：

```bash
vocabbar --version
vocabbar --help
```

### 直接下载 CLI（计划）

后续会提供各平台 CLI 二进制下载。下载后需要确保 `vocabbar` 在 `PATH` 中，或使用二进制的绝对路径执行。

```bash
vocabbar --version
```

### Pi extension 安装（计划）

Pi extension 发布后，用户只需要安装扩展包：

```bash
pi install npm:@vocabber/pi-extension
```

Pi extension 会依赖 `@vocabber/cli`，安装扩展时会自动获得 CLI。当前 Pi extension 仍在开发中。

## 初始化

首次使用先初始化本地数据库：

```bash
vocabbar init
```

默认数据库位置由系统决定。macOS 上通常位于：

```text
~/Library/Application Support/vocabbar/vocabbar.db
```

## 导入词库

导入 Qwerty Learner JSON：

```bash
vocabbar import qwerty assets/dicts/qwerty-learner/dicts/CET4_T.json --deck cet4 --name "CET-4"
```

导入 CSV：

```bash
vocabbar import csv ./words.csv --deck custom --name "Custom"
```

导入 JSONL：

```bash
vocabbar import jsonl ./deck.jsonl --deck custom --name "Custom"
```

导入 Anki TSV：

```bash
vocabbar import anki-tsv ./anki.txt --deck anki --name "Anki"
```

查看词库和卡片：

```bash
vocabbar deck list
vocabbar card list --deck cet4
```

## 学习命令

查看当前卡片：

```bash
vocabbar current --json
```

选择下一张卡片：

```bash
vocabbar next --json
```

评分：

```bash
vocabbar rate again --json
vocabbar rate hard --json
vocabbar rate good --json
vocabbar rate easy --json
```

`current` 和 `next` 不写 review log。只有 `rate again|hard|good|easy` 会更新学习状态。

## 输出格式

前端集成应使用 JSON：

```bash
vocabbar current --json
```

人工查看可以使用 plain/compact/status：

```bash
vocabbar current --format plain
vocabbar current --format compact
vocabbar current --format status
```

## 当前限制

- 暂不提供词库编辑器。
- 多词库 active deck 选择正在规划中。
- Pi extension、Claude Code adapter、Codex adapter 尚未发布。
