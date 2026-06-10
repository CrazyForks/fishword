import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const packageDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(packageDir, "../..");

const platformPackages = {
  "darwin-arm64": "@vocabber/cli-darwin-arm64",
  "darwin-x64": "@vocabber/cli-darwin-x64",
  "linux-arm64": "@vocabber/cli-linux-arm64",
  "linux-x64": "@vocabber/cli-linux-x64",
  "win32-x64": "@vocabber/cli-win32-x64"
};

function executableName() {
  return process.platform === "win32" ? "vocabbar.exe" : "vocabbar";
}

function devBinaryPath() {
  return join(repoRoot, "target", "debug", executableName());
}

function platformPackageName() {
  return platformPackages[`${process.platform}-${process.arch}`];
}

function platformBinaryPath() {
  const packageName = platformPackageName();
  if (!packageName) {
    return undefined;
  }

  try {
    const packageJsonPath = require.resolve(`${packageName}/package.json`);
    return join(dirname(packageJsonPath), "bin", executableName());
  } catch {
    return undefined;
  }
}

export function resolveVocabbarPath() {
  if (process.env.VOCABBAR_CLI_PATH) {
    return resolve(process.env.VOCABBAR_CLI_PATH);
  }

  const devPath = devBinaryPath();
  if (existsSync(devPath)) {
    return devPath;
  }

  const packagedPath = platformBinaryPath();
  if (packagedPath && existsSync(packagedPath)) {
    return packagedPath;
  }

  const platformName = `${process.platform}-${process.arch}`;
  const packageName = platformPackageName();
  const expected = packageName ?? "no supported @vocabber/cli-* package";
  throw new Error(
    [
      `Cannot find vocabbar CLI for ${platformName}.`,
      `Expected ${expected}.`,
      "For local development, run `pnpm dev:cli` from the repository root.",
      "You can also set VOCABBAR_CLI_PATH to a custom vocabbar binary."
    ].join(" ")
  );
}

export const vocabbarPath = resolveVocabbarPath();
