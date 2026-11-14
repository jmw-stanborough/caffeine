import { describe, expect, it } from "vitest";
import { toErrorMessage } from "./errors";

describe("toErrorMessage", () => {
  it("extracts message from Error instances", () => {
    expect(toErrorMessage(new Error("network failure"))).toBe("network failure");
  });

  it("returns string primitives directly", () => {
    expect(toErrorMessage("command rejected")).toBe("command rejected");
  });

  it("converts numbers and booleans to string", () => {
    expect(toErrorMessage(404)).toBe("404");
    expect(toErrorMessage(false)).toBe("false");
  });

  it("handles null and undefined safely", () => {
    expect(toErrorMessage(null)).toBe("null");
    expect(toErrorMessage(undefined)).toBe("undefined");
  });

  it("converts objects to string representations", () => {
    expect(toErrorMessage({ message: "ignored" })).toBe("[object Object]");
  });
});
