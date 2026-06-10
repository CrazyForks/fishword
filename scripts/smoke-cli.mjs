import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { vocabbarPath } from "../packages/cli/index.js";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const home = mkdtempSync(join(tmpdir(), "vocabbar-smoke-"));
const env = { ...process.env, HOME: home };

function run(args, options = {}) {
  const result = spawnSync(vocabbarPath, args, {
    cwd: repoRoot,
    env,
    encoding: "utf8"
  });

  if (result.status !== 0) {
    throw new Error(
      [
        `vocabbar ${args.join(" ")} failed with exit code ${result.status}`,
        result.stdout,
        result.stderr
      ]
        .filter(Boolean)
        .join("\n")
    );
  }

  if (options.json) {
    return JSON.parse(result.stdout);
  }

  return result.stdout;
}

try {
  run(["init"]);
  run([
    "import",
    "qwerty",
    "crates/vocabbar-core/fixtures/qwerty_cet4_sample.json",
    "--deck",
    "smoke",
    "--name",
    "Smoke"
  ]);

  const current = run(["current", "--json"], { json: true });
  if (!current.card?.term) {
    throw new Error("current --json did not return a card term");
  }

  const next = run(["next", "--json"], { json: true });
  if (!next.card?.term) {
    throw new Error("next --json did not return a card term");
  }

  const rated = run(["rate", "good", "--json"], { json: true });
  if (rated.review?.rating !== "good") {
    throw new Error("rate good --json did not return a good review");
  }

  console.log(`smoke:cli ok (${current.card.term})`);
} finally {
  rmSync(home, { recursive: true, force: true });
}
