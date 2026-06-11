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

## 导入方法

```bash
fishword import jsonl assets/dicts/kajweb/cet4.jsonl --deck cet4 --name "CET-4"
fishword import jsonl assets/dicts/kajweb/cet6.jsonl --deck cet6 --name "CET-6"
fishword import jsonl assets/dicts/kajweb/kaoyan.jsonl --deck kaoyan --name "考研英语"
fishword import jsonl assets/dicts/kajweb/ielts.jsonl --deck ielts --name "IELTS"
fishword import jsonl assets/dicts/kajweb/toefl.jsonl --deck toefl --name "TOEFL"
fishword import jsonl assets/dicts/kajweb/sat.jsonl --deck sat --name "SAT"
fishword import jsonl assets/dicts/kajweb/gre.jsonl --deck gre --name "GRE"
fishword import jsonl assets/dicts/kajweb/gmat.jsonl --deck gmat --name "GMAT"
```

## 数据来源与协议

上游仓库未提供 LICENSE 文件，数据来自有道词典 API。建议仅用于个人学习用途，不作商业分发。

转换脚本：`scripts/kajweb_to_jsonl.py`
