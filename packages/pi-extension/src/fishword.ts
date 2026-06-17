import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fishwordPath } from "@fishword/cli";
import type { CardResponse } from "./types.ts";

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

export async function runFishwordText(args: string[]): Promise<string> {
  const { stdout } = await execAsync(fishwordPath, args);
  return stdout.trim();
}

export function isErrorResponse(res: Record<string, unknown>): boolean {
  return res["schema"] === "fishword.protocol.error.v1";
}

export function getErrorCode(res: Record<string, unknown>): string | undefined {
  return (res["error"] as { code?: string })?.code;
}

export function getErrorMessage(res: Record<string, unknown>): string | undefined {
  return (res["error"] as { message?: string })?.message;
}

export function describeFishwordError(err: unknown): string {
  if (err instanceof Error && err.message.trim()) {
    const execErr = err as Error & { stderr?: string };
    return execErr.stderr?.trim() || err.message;
  }
  return "unknown error";
}

export function parseCardResponse(res: Record<string, unknown>): CardResponse {
  return res as CardResponse;
}
