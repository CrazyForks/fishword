# @fishword/pi-extension

Fishword 的 Pi 编程助手扩展，在编程时内嵌间隔重复词汇学习。

[GitHub 仓库](https://github.com/Chenggou1/fishword)

[观看演示视频](https://media.githubusercontent.com/media/Chenggou1/fishword/main/docs/videos/pi-extension-demo.mp4)

## 安装

```
pi install npm:@fishword/pi-extension
```

重启 Pi 后会通过 Fishword catalog 自动下载 CET-4 / CET-6 / TOEFL 三个默认词库，无需手动导入词表。

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+F` | 隐藏或唤起 Fishword UI |
| `Ctrl+Shift+I` | 打开详情面板（音标、词性、释义、例句） |
| `Ctrl+Shift+G` | 评分：good（记住了） |
| `Ctrl+Shift+H` | 评分：hard（有点难） |
| `Ctrl+Shift+A` | 评分：again（没记住） |
| `Ctrl+Shift+E` | 评分：easy（轻松） |

评分快捷键在词卡视图和详情面板内均有效。详情面板内还额外支持 `G` / `H` / `A` / `E` 单键评分。

## Slash 命令

| 命令 | 功能 |
|------|------|
| `/fw` | 隐藏或唤起 Fishword UI |
| `/fw-detail` | 打开当前单词的详情面板 |
| `/fw-stats` | 查看今日进度和 7 日学习趋势 |
| `/fw-deck` | 切换激活词库 |
| `/fw-good` | 评分：good（记住了） |
| `/fw-hard` | 评分：hard（有点难） |
| `/fw-again` | 评分：again（没记住） |
| `/fw-easy` | 评分：easy（轻松） |

## 许可证

GPL-3.0-only
