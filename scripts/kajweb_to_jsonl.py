# /// script
# requires-python = ">=3.11"
# dependencies = ["requests"]
# ///
"""
将 kajweb/dict 词书转换为 fishword DeckCardV1 JSONL 格式。

用法：
    uv run scripts/kajweb_to_jsonl.py --book CET4 -o assets/dicts/kajweb/cet4.jsonl
    uv run scripts/kajweb_to_jsonl.py --book CET6 -o assets/dicts/kajweb/cet6.jsonl
    uv run scripts/kajweb_to_jsonl.py --book KaoYan -o assets/dicts/kajweb/kaoyao.jsonl
    uv run scripts/kajweb_to_jsonl.py --book TOEFL -o assets/dicts/kajweb/toefl.jsonl
    uv run scripts/kajweb_to_jsonl.py --book GRE -o assets/dicts/kajweb/gre.jsonl
    uv run scripts/kajweb_to_jsonl.py --book IELTS -o assets/dicts/kajweb/ielts.jsonl
    uv run scripts/kajweb_to_jsonl.py --book SAT -o assets/dicts/kajweb/sat.jsonl
    uv run scripts/kajweb_to_jsonl.py --book GMAT -o assets/dicts/kajweb/gmat.jsonl

数据来源：https://github.com/kajweb/dict
"""

import argparse
import io
import json
import sys
import zipfile
from pathlib import Path

import requests

# kajweb/dict GitHub raw base
RAW_BASE = "https://raw.githubusercontent.com/kajweb/dict/master"

# book_id -> list of ZIP filenames in the repo
BOOK_ZIPS: dict[str, list[str]] = {
    "CET4":   ["1521164649209_CET4_1.zip", "1521164635506_CET4_2.zip", "1521164643060_CET4_3.zip"],
    "CET6":   ["1521164668667_CET6_1.zip", "1524052554766_CET6_2.zip", "1521164633851_CET6_3.zip"],
    "KaoYan": ["1521164669833_KaoYan_1.zip", "1521164654696_KaoYan_2.zip", "1521164658897_KaoYan_3.zip"],
    "TOEFL":  ["1521164640451_TOEFL_2.zip", "1521164667985_TOEFL_3.zip"],
    "GRE":    ["1521164637271_GRE_2.zip", "1521164677706_GRE_3.zip"],
    "IELTS":  ["1521164657744_IELTS_2.zip", "1521164666922_IELTS_3.zip"],
    "SAT":    ["1521164670910_SAT_2.zip", "1521164636496_SAT_3.zip"],
    "GMAT":   ["1521164662073_GMAT_2.zip", "1521164672691_GMAT_3.zip"],
}

TAG_MAP: dict[str, str] = {
    "CET4": "cet4",
    "CET6": "cet6",
    "KaoYan": "kaoyan",
    "TOEFL": "toefl",
    "GRE": "gre",
    "IELTS": "ielts",
    "SAT": "sat",
    "GMAT": "gmat",
}


def fetch_zip(url: str) -> bytes:
    resp = requests.get(url, timeout=30)
    resp.raise_for_status()
    return resp.content


def parse_ndjson_from_zip(data: bytes) -> list[dict]:
    """从 ZIP 中读取唯一的 JSON 文件，按行解析 NDJSON。"""
    entries = []
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        for name in zf.namelist():
            if name.endswith(".json"):
                text = zf.read(name).decode("utf-8")
                for line in text.splitlines():
                    line = line.strip()
                    if line:
                        entries.append(json.loads(line))
    return entries


def entry_to_card(entry: dict, tag: str) -> dict | None:
    word = entry.get("headWord", "").strip()
    if not word:
        return None

    content = entry.get("content", {}).get("word", {}).get("content", {})
    trans_list = content.get("trans", [])
    if not trans_list:
        return None

    # 取第一条例句
    sentences = content.get("sentence", {}).get("sentences", [])
    example = sentences[0].get("sContent", "").strip() if sentences else None
    if not example:
        example = None

    meanings = []
    for i, t in enumerate(trans_list):
        pos = t.get("pos", "").strip()
        text = t.get("tranCn", "").strip()
        if not text:
            continue
        meanings.append({
            "lang": pos if pos else "zh-CN",
            "text": text,
            # 例句只挂在第一个 meaning 上
            **({"example": example} if i == 0 and example else {}),
        })

    if not meanings:
        return None

    card: dict = {
        "term": word,
        "language": "en",
        "meanings": meanings,
        "tags": [tag],
        "source": {"name": "kajweb/dict"},
    }

    us = content.get("usphone", "").strip()
    uk = content.get("ukphone", "").strip()
    if us or uk:
        card["pronunciation"] = {}
        if us:
            card["pronunciation"]["us"] = us
        if uk:
            card["pronunciation"]["uk"] = uk

    return card


def convert_book(book: str) -> list[dict]:
    zips = BOOK_ZIPS.get(book)
    if not zips:
        raise ValueError(f"未知词书：{book}，可选：{', '.join(BOOK_ZIPS)}")

    tag = TAG_MAP[book]
    seen: set[str] = set()
    cards = []

    for zip_name in zips:
        url = f"{RAW_BASE}/book/{zip_name}"
        print(f"  下载 {zip_name} ...", file=sys.stderr)
        data = fetch_zip(url)
        entries = parse_ndjson_from_zip(data)
        for entry in entries:
            card = entry_to_card(entry, tag)
            if card and card["term"] not in seen:
                seen.add(card["term"])
                cards.append(card)

    return cards


def main() -> None:
    parser = argparse.ArgumentParser(description="kajweb/dict → fishword JSONL")
    parser.add_argument("--book", required=True, choices=list(BOOK_ZIPS), help="词书名称")
    parser.add_argument("-o", "--output", help="输出文件路径（默认 stdout）")
    args = parser.parse_args()

    print(f"转换词书：{args.book}", file=sys.stderr)
    cards = convert_book(args.book)
    print(f"共 {len(cards)} 个词条", file=sys.stderr)

    lines = [json.dumps(c, ensure_ascii=False) for c in cards]
    output = "\n".join(lines) + "\n"

    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        Path(args.output).write_text(output, encoding="utf-8")
        print(f"已写入 {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(output)


if __name__ == "__main__":
    main()
