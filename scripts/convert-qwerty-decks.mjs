#!/usr/bin/env node
/**
 * Convert selected qwerty-learner JSON dicts and kajweb JSONL files
 * into fishword.deck.v1 JSONL format, then generate a catalog.json index.
 *
 * Output: dist/catalog/
 *   catalog.json          — index of all published decks
 *   <source-id>-<slug>.jsonl — one fishword.deck.v1 JSONL file per deck
 *
 * Usage: node scripts/convert-qwerty-decks.mjs
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const QWERTY_DIR = path.join(ROOT, "assets/dicts/qwerty-learner/dicts");
const KAJWEB_DIR = path.join(ROOT, "assets/dicts/kajweb");
const OUT_DIR = path.join(ROOT, "dist/catalog");

// Selected qwerty source files and their catalog metadata.
// Only included when there is no higher-quality kajweb equivalent —
// kajweb already covers cet4/cet6/toefl with richer data (POS, examples),
// so the qwerty versions of those are intentionally omitted to avoid
// publishing duplicate, lower-quality decks. Curate for quality, not breadth.
const QWERTY_DECKS = [
  {
    srcFile: "itVocabulary.json",
    sourceId: "qwerty",
    slug: "programmer-english",
    name: "程序员英语",
    description: "开发者常见计算机英语词汇，适合 Coding Agent 和日常开发场景",
    tags: ["programming", "developer", "computer-english", "zh"],
  },
  {
    srcFile: "linux-command.json",
    sourceId: "qwerty",
    slug: "linux-commands",
    name: "Linux Commands",
    description: "Linux 常用命令词库，适合终端、服务器和 CLI 场景",
    tags: ["linux", "cli", "developer"],
  },
  {
    srcFile: "ai_machine_learning.json",
    sourceId: "qwerty",
    slug: "ai-machine-learning",
    name: "AI / Machine Learning",
    description: "AI 与机器学习常见术语，适合 Coding Agent 和 AI 应用开发场景",
    tags: ["ai", "machine-learning", "developer"],
  },
  {
    srcFile: "IELTS_3_T.json",
    sourceId: "qwerty",
    slug: "ielts",
    name: "IELTS",
    description: "雅思核心词汇，来自 Qwerty Learner",
    tags: ["ielts", "exam", "zh"],
  },
  {
    srcFile: "GRE_3_T.json",
    sourceId: "qwerty",
    slug: "gre",
    name: "GRE",
    description: "GRE 核心词汇，来自 Qwerty Learner",
    tags: ["gre", "exam", "zh"],
  },
  {
    srcFile: "SAT_3_T.json",
    sourceId: "qwerty",
    slug: "sat",
    name: "SAT",
    description: "SAT 核心词汇，来自 Qwerty Learner",
    tags: ["sat", "exam", "zh"],
  },
  {
    srcFile: "Oxford3000.json",
    sourceId: "qwerty",
    slug: "oxford3000",
    name: "Oxford 3000",
    description: "牛津 3000 核心词汇，来自 Qwerty Learner",
    tags: ["oxford", "reference"],
  },
  {
    srcFile: "Oxford5000.json",
    sourceId: "qwerty",
    slug: "oxford5000",
    name: "Oxford 5000",
    description: "牛津 5000 核心词汇，来自 Qwerty Learner",
    tags: ["oxford", "reference"],
  },
];

// kajweb files are already in fishword.deck.v1 JSONL format.
const KAJWEB_DECKS = [
  {
    srcFile: "cet4.jsonl",
    sourceId: "kajweb",
    slug: "cet4",
    name: "CET-4",
    description: "大学英语四级词汇，含词性、例句",
    tags: ["cet4", "exam", "zh"],
  },
  {
    srcFile: "cet6.jsonl",
    sourceId: "kajweb",
    slug: "cet6",
    name: "CET-6",
    description: "大学英语六级词汇，含词性、例句",
    tags: ["cet6", "exam", "zh"],
  },
  {
    srcFile: "toefl.jsonl",
    sourceId: "kajweb",
    slug: "toefl",
    name: "TOEFL",
    description: "托福词汇，含词性、例句",
    tags: ["toefl", "exam", "zh"],
  },
];

// ── Conversion helpers ───────────────────────────────────────────────────────

/**
 * Convert a single qwerty word object to a fishword.deck.v1 entry.
 * @param {{ name: string, trans: string[], usphone?: string, ukphone?: string }} word
 * @param {string[]} tags
 * @returns {object}
 */
function qwertyWordToDeckEntry(word, tags) {
  const entry = {
    term: word.name,
    language: "en",
    meanings: (word.trans ?? []).map((t) => ({ lang: "zh-CN", text: t })),
    tags,
    source: { name: "qwerty-learner", license: "GPL-3.0" },
  };

  const pronunciation = {};
  if (word.usphone) pronunciation.us = word.usphone;
  if (word.ukphone) pronunciation.uk = word.ukphone;
  if (Object.keys(pronunciation).length > 0) entry.pronunciation = pronunciation;

  return entry;
}

function mergeQwertyEntry(existing, incoming) {
  for (const meaning of incoming.meanings) {
    if (
      !existing.meanings.some(
        (item) => item.lang === meaning.lang && item.text === meaning.text,
      )
    ) {
      existing.meanings.push(meaning);
    }
  }

  for (const tag of incoming.tags) {
    if (!existing.tags.includes(tag)) {
      existing.tags.push(tag);
    }
  }

  if (incoming.pronunciation) {
    existing.pronunciation ??= {};
    existing.pronunciation.us ??= incoming.pronunciation.us;
    existing.pronunciation.uk ??= incoming.pronunciation.uk;
    if (Object.keys(existing.pronunciation).length === 0) {
      delete existing.pronunciation;
    }
  }

  return existing;
}

function dedupeQwertyEntries(words, tags) {
  const byTerm = new Map();
  for (const word of words) {
    const entry = qwertyWordToDeckEntry(word, tags);
    const existing = byTerm.get(entry.term);
    if (existing) {
      mergeQwertyEntry(existing, entry);
    } else {
      byTerm.set(entry.term, entry);
    }
  }
  return [...byTerm.values()];
}

function jsonlLines(entries) {
  return entries.map((entry) => JSON.stringify(entry));
}

function parseJsonlEntries(content, label) {
  return content
    .split("\n")
    .filter((line) => line.trim())
    .map((line, index) => {
      try {
        return JSON.parse(line);
      } catch (error) {
        throw new Error(`${label} has invalid JSON on line ${index + 1}: ${error.message}`);
      }
    });
}

function assertNoDuplicateTerms(entries, label) {
  const seen = new Set();
  const duplicates = new Set();
  for (const entry of entries) {
    if (seen.has(entry.term)) {
      duplicates.add(entry.term);
    }
    seen.add(entry.term);
  }

  if (duplicates.size > 0) {
    throw new Error(
      `${label} still contains duplicate terms: ${[...duplicates].slice(0, 10).join(", ")}`,
    );
  }
}

function catalogId(deck) {
  return `${deck.sourceId}:${deck.slug}`;
}

function deckFilename(deck) {
  return `${deck.sourceId}-${deck.slug}.jsonl`;
}

// ── Main ─────────────────────────────────────────────────────────────────────

fs.rmSync(OUT_DIR, { recursive: true, force: true });
fs.mkdirSync(OUT_DIR, { recursive: true });

const catalogDecks = [];

// 1. Convert qwerty JSON files
for (const deck of QWERTY_DECKS) {
  const srcPath = path.join(QWERTY_DIR, deck.srcFile);
  if (!fs.existsSync(srcPath)) {
    console.warn(`[skip] not found: ${srcPath}`);
    continue;
  }

  const words = JSON.parse(fs.readFileSync(srcPath, "utf8"));
  const baseTag = deck.slug;
  const tags = Array.from(new Set([baseTag, ...deck.tags]));

  const entries = dedupeQwertyEntries(words, tags);
  assertNoDuplicateTerms(entries, catalogId(deck));
  const lines = jsonlLines(entries);
  const filename = deckFilename(deck);
  const outPath = path.join(OUT_DIR, filename);
  fs.writeFileSync(outPath, lines.join("\n") + "\n", "utf8");

  const stat = fs.statSync(outPath);
  catalogDecks.push({
    id: catalogId(deck),
    slug: deck.slug,
    source_id: deck.sourceId,
    name: deck.name,
    description: deck.description,
    language: "en",
    word_count: lines.length,
    tags: deck.tags,
    source: { name: "qwerty-learner", license: "GPL-3.0" },
    url: `{BASE_URL}/${filename}`,
    size_bytes: stat.size,
  });

  const removed = words.length - entries.length;
  const dedupeSuffix = removed > 0 ? ` (${removed} duplicate rows merged)` : "";
  console.log(`[qwerty] ${catalogId(deck)}: ${lines.length} words${dedupeSuffix} → ${outPath}`);
}

// 2. Copy kajweb JSONL files (already in fishword.deck.v1 format)
for (const deck of KAJWEB_DECKS) {
  const srcPath = path.join(KAJWEB_DIR, deck.srcFile);
  if (!fs.existsSync(srcPath)) {
    console.warn(`[skip] not found: ${srcPath}`);
    continue;
  }

  const filename = deckFilename(deck);
  const outPath = path.join(OUT_DIR, filename);
  fs.copyFileSync(srcPath, outPath);

  const content = fs.readFileSync(outPath, "utf8");
  const entries = parseJsonlEntries(content, catalogId(deck));
  assertNoDuplicateTerms(entries, catalogId(deck));
  const wordCount = entries.length;
  const stat = fs.statSync(outPath);

  catalogDecks.push({
    id: catalogId(deck),
    slug: deck.slug,
    source_id: deck.sourceId,
    name: deck.name,
    description: deck.description,
    language: "en",
    word_count: wordCount,
    tags: deck.tags,
    source: { name: "kajweb/dict" },
    url: `{BASE_URL}/${filename}`,
    size_bytes: stat.size,
  });

  console.log(`[kajweb] ${catalogId(deck)}: ${wordCount} words → ${outPath}`);
}

// 3. Generate catalog.json
// BASE_URL is replaced at deploy time by the publish workflow.
// When deploying to GitHub Pages the URL is known, so we patch it in the workflow.
// During local development the placeholder remains so catalog.json is still readable.
const BASE_URL = process.env.FISHWORD_CATALOG_BASE_URL
  ?? "https://chenggou1.github.io/fishword/catalog";

const resolvedDecks = catalogDecks.map((d) => ({
  ...d,
  url: d.url.replace("{BASE_URL}", BASE_URL),
}));

const catalog = {
  schema: "fishword.catalog.v2",
  updated_at: new Date().toISOString(),
  decks: resolvedDecks,
};

const catalogPath = path.join(OUT_DIR, "catalog.json");
fs.writeFileSync(catalogPath, JSON.stringify(catalog, null, 2), "utf8");
console.log(`\ncatalog.json: ${resolvedDecks.length} decks → ${catalogPath}`);
