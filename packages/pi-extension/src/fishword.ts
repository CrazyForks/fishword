import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fishwordPath } from "@fishword/cli";
import type { Card } from "./types";

const execAsync = promisify(execFile);

export async function runFishword(args: string[]): Promise<Record<string, unknown>> {
  try {
    const { stdout } = await execAsync(fishwordPath, args);
    return JSON.parse(stdout.trim()) as Record<string, unknown>;
  } catch (err: unknown) {
    const execErr = err as { stdout?: string };
    if (execErr.stdout) {
      try {
        return JSON.parse(execErr.stdout.trim()) as Record<string, unknown>;
      } catch {
        // Fall through to the original error.
      }
    }
    throw err;
  }
}

export function isErrorResponse(res: Record<string, unknown>): boolean {
  return res["schema"] === "fishword.protocol.error.v1";
}

export function getErrorCode(res: Record<string, unknown>): string | undefined {
  return (res["error"] as { code?: string })?.code;
}

export function parseCard(res: Record<string, unknown>): Card {
  return res["card"] as Card;
}
