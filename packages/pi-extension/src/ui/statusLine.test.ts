import { describe, expect, it } from "vitest";
import type { StatusResponse } from "../types.ts";
import { formatStatusLine, formatStatusLineMessage } from "./statusLine.ts";

function makeStatus(overrides: Partial<StatusResponse> = {}): StatusResponse {
  return {
    schema: "fishword.protocol.status.v1",
    deck: { id: "kajweb:cet4", name: "CET4", db_id: 1 },
    mode: "review",
    today: { due: 5, new_remaining: 3, new_today: 3, reviewed: 2 },
    display: { plain: "", compact: "", statusline: "" },
    next_action: { kind: "review", label: "Review", command: "fw" },
    ...overrides,
  };
}

// Strip ANSI escape codes for readable assertions.
function strip(s: string): string {
  return s.replace(/\x1b\[[0-9;]*m/g, "");
}

describe("formatStatusLine", () => {
  it("includes deck name", () => {
    const result = strip(formatStatusLine(makeStatus()));
    expect(result).toContain("CET4");
  });

  it("includes counts in review mode", () => {
    const result = strip(formatStatusLine(makeStatus({ today: { due: 5, new_remaining: 3, new_today: 3, reviewed: 2 } })));
    expect(result).toContain("5");
    expect(result).toContain("3");
    expect(result).toContain("2");
  });

  it("shows Done label in complete mode", () => {
    const result = strip(formatStatusLine(makeStatus({ mode: "complete" })));
    expect(result).toContain("Done");
  });

  it("shows No cards in empty mode", () => {
    const result = strip(formatStatusLine(makeStatus({ mode: "empty" })));
    expect(result).toContain("No cards");
  });
});

describe("formatStatusLineMessage", () => {
  it("no-deck includes 'No deck'", () => {
    const result = strip(formatStatusLineMessage("no-deck"));
    expect(result).toContain("No deck");
  });

  it("unavailable includes 'Unavailable'", () => {
    const result = strip(formatStatusLineMessage("unavailable"));
    expect(result).toContain("Unavailable");
  });

  it("both include Fishword prefix", () => {
    expect(strip(formatStatusLineMessage("no-deck"))).toContain("Fishword");
    expect(strip(formatStatusLineMessage("unavailable"))).toContain("Fishword");
  });
});
