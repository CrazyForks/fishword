import { describe, expect, it } from "vitest";
import type { Card } from "../types.ts";
import { formatMeaning, formatPercent, formatPhonetic } from "./text.ts";

function makeCard(overrides: Partial<Card> = {}): Card {
  return {
    id: "1",
    term: "test",
    language: "en",
    meanings: [],
    deck: { id: "kajweb:cet4", name: "CET4", db_id: 1 },
    tags: [],
    ...overrides,
  };
}

describe("formatPhonetic", () => {
  it("returns US phonetic wrapped in slashes", () => {
    expect(formatPhonetic(makeCard({ phonetic: { us: "tɛst" } }))).toBe("/tɛst/");
  });

  it("falls back to UK when US is absent", () => {
    expect(formatPhonetic(makeCard({ phonetic: { uk: "tɛst" } }))).toBe("/tɛst/");
  });

  it("strips existing slashes from raw value", () => {
    expect(formatPhonetic(makeCard({ phonetic: { us: "/tɛst/" } }))).toBe("/tɛst/");
  });

  it("returns empty string when phonetic is absent", () => {
    expect(formatPhonetic(makeCard())).toBe("");
  });

  it("returns empty string when phonetic values are null", () => {
    expect(formatPhonetic(makeCard({ phonetic: { us: null, uk: null } }))).toBe("");
  });
});

describe("formatMeaning", () => {
  it("joins string meanings with semicolon", () => {
    expect(formatMeaning(makeCard({ meanings: ["n. 考试", "v. 测试"] }))).toBe("n. 考试；v. 测试");
  });

  it("formats structured meanings as 'pos. definition'", () => {
    expect(
      formatMeaning(makeCard({ meanings: [{ part_of_speech: "n.", definition: "test" }] })),
    ).toBe("n.. test");
  });

  it("handles missing part_of_speech", () => {
    expect(
      formatMeaning(makeCard({ meanings: [{ part_of_speech: "", definition: "test" }] })),
    ).toBe("test");
  });

  it("returns empty string for empty meanings", () => {
    expect(formatMeaning(makeCard({ meanings: [] }))).toBe("");
  });

  it("normalises extra whitespace", () => {
    expect(formatMeaning(makeCard({ meanings: ["  n.   test  "] }))).toBe("n. test");
  });
});

describe("formatPercent", () => {
  it("formats 1.0 as 100%", () => {
    expect(formatPercent(1.0)).toBe("100%");
  });

  it("formats 0.5 as 50%", () => {
    expect(formatPercent(0.5)).toBe("50%");
  });

  it("rounds to nearest integer", () => {
    expect(formatPercent(0.756)).toBe("76%");
  });

  it("returns -- for null", () => {
    expect(formatPercent(null)).toBe("--");
  });

  it("formats 0 as 0%", () => {
    expect(formatPercent(0)).toBe("0%");
  });
});
