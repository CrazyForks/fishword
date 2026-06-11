import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");

const defaultDecks = ["cet4.jsonl", "cet6.jsonl", "toefl.jsonl"];
const sourceDir = resolve(repoRoot, "assets/dicts/kajweb");
const targetDir = resolve(repoRoot, "packages/pi-extension/assets/dicts/kajweb");

mkdirSync(targetDir, { recursive: true });

for (const fileName of defaultDecks) {
  copyFileSync(resolve(sourceDir, fileName), resolve(targetDir, fileName));
}
