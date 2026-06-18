import { describe, expect, it } from "vitest";
import { describeFishwordError, getErrorCode, getErrorMessage, isErrorResponse } from "./fishword.ts";

const ERROR_RESPONSE = {
  schema: "fishword.protocol.error.v1",
  error: { code: "no_active_deck", message: "No active deck selected" },
};

const CARD_RESPONSE = {
  schema: "fishword.protocol.current.v1",
  card: {},
};

describe("isErrorResponse", () => {
  it("returns true for error schema", () => {
    expect(isErrorResponse(ERROR_RESPONSE)).toBe(true);
  });

  it("returns false for non-error schema", () => {
    expect(isErrorResponse(CARD_RESPONSE)).toBe(false);
  });

  it("returns false for empty object", () => {
    expect(isErrorResponse({})).toBe(false);
  });
});

describe("getErrorCode", () => {
  it("returns code from error field", () => {
    expect(getErrorCode(ERROR_RESPONSE)).toBe("no_active_deck");
  });

  it("returns undefined when no error field", () => {
    expect(getErrorCode({})).toBeUndefined();
  });

  it("returns undefined when error field has no code", () => {
    expect(getErrorCode({ error: {} })).toBeUndefined();
  });
});

describe("getErrorMessage", () => {
  it("returns message from error field", () => {
    expect(getErrorMessage(ERROR_RESPONSE)).toBe("No active deck selected");
  });

  it("returns undefined when no error field", () => {
    expect(getErrorMessage({})).toBeUndefined();
  });
});

describe("describeFishwordError", () => {
  it("returns stderr when present", () => {
    const err = Object.assign(new Error("ignored"), { stderr: "  fishword: fatal\n" });
    expect(describeFishwordError(err)).toBe("fishword: fatal");
  });

  it("falls back to message when stderr is absent", () => {
    const err = new Error("something went wrong");
    expect(describeFishwordError(err)).toBe("something went wrong");
  });

  it("falls back to message when stderr is empty", () => {
    const err = Object.assign(new Error("fallback"), { stderr: "" });
    expect(describeFishwordError(err)).toBe("fallback");
  });

  it("returns 'unknown error' for non-Error values", () => {
    expect(describeFishwordError("oops")).toBe("unknown error");
    expect(describeFishwordError(null)).toBe("unknown error");
  });
});
