# kajweb/dict 词库

来自 [kajweb/dict](https://github.com/kajweb/dict)，专为中国英语学习者设计，每个词条包含中文释义、词性和英文例句。

## 词库列表

| 文件 | 词书 | 词条数 |
|------|------|--------|
| `cet4.jsonl` | CET-4（大学英语四级） | 4544 |
| `cet6.jsonl` | CET-6（大学英语六级） | 3992 |
| `kaoyan.jsonl` | 考研英语 | 5057 |
| `ielts.jsonl` | IELTS（雅思） | 5275 |
| `toefl.jsonl` | TOEFL（托福） | 10377 |
| `sat.jsonl` | SAT | 4464 |
| `gre.jsonl` | GRE | 9984 |
| `gmat.jsonl` | GMAT | 3312 |

## 生成方法

```bash
uv run scripts/kajweb_to_jsonl.py --book CET4 -o assets/dicts/kajweb/cet4.jsonl
uv run scripts/kajweb_to_jsonl.py --book CET6 -o assets/dicts/kajweb/cet6.jsonl
uv run scripts/kajweb_to_jsonl.py --book KaoYan -o assets/dicts/kajweb/kaoyan.jsonl
uv run scripts/kajweb_to_jsonl.py --book IELTS -o assets/dicts/kajweb/ielts.jsonl
uv run scripts/kajweb_to_jsonl.py --book TOEFL -o assets/dicts/kajweb/toefl.jsonl
uv run scripts/kajweb_to_jsonl.py --book SAT -o assets/dicts/kajweb/sat.jsonl
uv run scripts/kajweb_to_jsonl.py --book GRE -o assets/dicts/kajweb/gre.jsonl
uv run scripts/kajweb_to_jsonl.py --book GMAT -o assets/dicts/kajweb/gmat.jsonl
```

## 手动导入方法

```bash
fishword deck create CET-4 --description "大学英语四级"
fishword import jsonl assets/dicts/kajweb/cet4.jsonl --deck 1
```

## 数据来源与协议

上游仓库未提供 LICENSE 文件，数据来自有道词典 API。建议仅用于个人学习用途，不作商业分发。

转换脚本：`scripts/kajweb_to_jsonl.py`
