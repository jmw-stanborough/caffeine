import { describe, expect, it } from "vitest";
import { getEffectivePrinterStatus } from "./status";
import type { TonerSupply } from "./types";

function createToner(percent: number, color = "black"): TonerSupply {
  return {
    name: `${color} Toner`,
    color,
    level: percent,
    max_capacity: 100,
    percent,
    supply_type: "toner",
  };
}

describe("getEffectivePrinterStatus", () => {
  it("defaults to checking when state is undefined", () => {
    expect(getEffectivePrinterStatus(undefined)).toBe("checking");
  });

  it("preserves offline base status", () => {
    expect(getEffectivePrinterStatus({ status: "offline" })).toBe("offline");
  });

  it("prioritizes paper jam over online status", () => {
    expect(
      getEffectivePrinterStatus({
        status: "online",
        paperJam: true,
      }),
    ).toBe("offline");
  });

  it("prioritizes out of paper over online status", () => {
    expect(
      getEffectivePrinterStatus({
        status: "online",
        outOfPaper: true,
      }),
    ).toBe("offline");
  });

  it("marks empty primary toner as offline", () => {
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners: [createToner(0)],
      }),
    ).toBe("offline");
  });

  it("marks low toner as degraded when otherwise online", () => {
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners: [createToner(10, "cyan")],
      }),
    ).toBe("degraded");
  });

  it("returns online when all supplies and conditions are nominal", () => {
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners: [createToner(80)],
        paperJam: false,
        outOfPaper: false,
      }),
    ).toBe("online");
  });
});
