#!/usr/bin/env node
/**
 * Convert selected qwerty-learner JSON dicts and kajweb JSONL files
 * into fishword.deck.v1 JSONL format, then generate a catalog.json index.
 *
 * Output: dist/catalog/
 *   catalog.json          — index of all published decks
 *   <deck-id>.jsonl       — one fishword.deck.v1 JSONL file per deck
 *
 * Usage: node scripts/convert-qwerty-decks.mjs
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const QWERTY_DIR = path.join(ROOT, "assets/dicts/qwerty-learner/dicts");
const KAJWEB_DIR = path.join(
  ROOT,
  "packages/pi-extension/assets/dicts/kajweb"
);
const OUT_DIR = path.join(ROOT, "dist/catalog");

// Selected qwerty source files and their catalog metadata.
// Only the most widely used exam/reference lists are included.
const QWERTY_DECKS = [
  {
    srcFile: "CET4_T.json",
    id: "cet4-qwerty",
    name: "CET-4 (Qwerty Learner)",
    description: "大学英语四级核心词汇，来自 Qwerty Learner",
    tags: ["cet4", "exam", "zh"],
  },
  {
    srcFile: "CET6_T.json",
    id: "cet6-qwerty",
    name: "CET-6 (Qwerty Learner)",
    description: "大学英语六级核心词汇，来自 Qwerty Learner",
    tags: ["cet6", "exam", "zh"],
  },
  {
    srcFile: "IELTS_3_T.json",
    id: "ielts-qwerty",
    name: "IELTS (Qwerty Learner)",
    description: "雅思核心词汇，来自 Qwerty Learner",
    tags: ["ielts", "exam", "zh"],
  },
  {
    srcFile: "TOEFL_3_T.json",
    id: "toefl-qwerty",
    name: "TOEFL (Qwerty Learner)",
    description: "托福核心词汇，来自 Qwerty Learner",
    tags: ["toefl", "exam", "zh"],
  },
  {
    srcFile: "GRE_3_T.json",
    id: "gre-qwerty",
    name: "GRE (Qwerty Learner)",
    description: "GRE 核心词汇，来自 Qwerty Learner",
    tags: ["gre", "exam", "zh"],
  },
  {
    srcFile: "SAT_3_T.json",
    id: "sat-qwerty",
    name: "SAT (Qwerty Learner)",
    description: "SAT 核心词汇，来自 Qwerty Learner",
    tags: ["sat", "exam", "zh"],
  },
  {
    srcFile: "Oxford3000.json",
    id: "oxford3000",
    name: "Oxford 3000",
    description: "牛津 3000 核心词汇，来自 Qwerty Learner",
    tags: ["oxford", "reference"],
  },
  {
    srcFile: "Oxford5000.json",
    id: "oxford5000",
    name: "Oxford 5000",
    description: "牛津 5000 核心词汇，来自 Qwerty Learner",
    tags: ["oxford", "reference"],
  },
];

// kajweb files are already in fishword.deck.v1 JSONL format.
const KAJWEB_DECKS = [
  {
    srcFile: "cet4.jsonl",
    id: "cet4",
    name: "CET-4",
    description: "大学英语四级词汇，含词性、例句",
    tags: ["cet4", "exam", "zh"],
  },
  {
    srcFile: "cet6.jsonl",
    id: "cet6",
    name: "CET-6",
    description: "大学英语六级词汇，含词性、例句",
    tags: ["cet6", "exam", "zh"],
  },
  {
    srcFile: "toefl.jsonl",
    id: "toefl",
    name: "TOEFL",
    description: "托福词汇，含词性、例句",
    tags: ["toefl", "exam", "zh"],
  },
];

// ── Conversion helpers ───────────────────────────────────────────────────────

/**
 * Convert a single qwerty word object to a fishword.deck.v1 JSON string.
 * @param {{ name: string, trans: string[], usphone?: string, ukphone?: string }} word
 * @param {string[]} tags
 * @returns {string} JSON line
 */
function qwertyWordToJsonlLine(word, tags) {
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

  return JSON.stringify(entry);
}

// ── Main ─────────────────────────────────────────────────────────────────────

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
  // Derive base tag from id (strip -qwerty suffix if present)
  const baseTag = deck.id.replace(/-qwerty$/, "");
  const tags = Array.from(new Set([baseTag, ...deck.tags]));

  const lines = words.map((w) => qwertyWordToJsonlLine(w, tags));
  const outPath = path.join(OUT_DIR, `${deck.id}.jsonl`);
  fs.writeFileSync(outPath, lines.join("\n") + "\n", "utf8");

  const stat = fs.statSync(outPath);
  catalogDecks.push({
    id: deck.id,
    name: deck.name,
    description: deck.description,
    language: "en",
    word_count: lines.length,
    tags: deck.tags,
    source: { name: "qwerty-learner", license: "GPL-3.0" },
    url: `{BASE_URL}/${deck.id}.jsonl`,
    size_bytes: stat.size,
  });

  console.log(`[qwerty] ${deck.id}: ${lines.length} words → ${outPath}`);
}

// 2. Copy kajweb JSONL files (already in fishword.deck.v1 format)
for (const deck of KAJWEB_DECKS) {
  const srcPath = path.join(KAJWEB_DIR, deck.srcFile);
  if (!fs.existsSync(srcPath)) {
    console.warn(`[skip] not found: ${srcPath}`);
    continue;
  }

  const outPath = path.join(OUT_DIR, `${deck.id}.jsonl`);
  fs.copyFileSync(srcPath, outPath);

  const content = fs.readFileSync(outPath, "utf8");
  const wordCount = content.split("\n").filter((l) => l.trim()).length;
  const stat = fs.statSync(outPath);

  catalogDecks.push({
    id: deck.id,
    name: deck.name,
    description: deck.description,
    language: "en",
    word_count: wordCount,
    tags: deck.tags,
    source: { name: "kajweb/dict" },
    url: `{BASE_URL}/${deck.id}.jsonl`,
    size_bytes: stat.size,
  });

  console.log(`[kajweb] ${deck.id}: ${wordCount} words → ${outPath}`);
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
  schema: "fishword.catalog.v1",
  updated_at: new Date().toISOString(),
  decks: resolvedDecks,
};

const catalogPath = path.join(OUT_DIR, "catalog.json");
fs.writeFileSync(catalogPath, JSON.stringify(catalog, null, 2), "utf8");
console.log(`\ncatalog.json: ${resolvedDecks.length} decks → ${catalogPath}`);
