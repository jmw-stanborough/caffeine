import { describe, expect, it } from "vitest";
import { getEffectivePrinterStatus } from "./status";
import type { PrinterState, TonerSupply } from "./types";

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
    const toners: TonerSupply[] = [
      {
        name: "Black Toner",
        color: "black",
        level: 0,
        max_capacity: 100,
        percent: 0,
        supply_type: "toner",
      },
    ];
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners,
      }),
    ).toBe("offline");
  });

  it("marks low toner as degraded when otherwise online", () => {
    const toners: TonerSupply[] = [
      {
        name: "Cyan Toner",
        color: "cyan",
        level: 10,
        max_capacity: 100,
        percent: 10,
        supply_type: "toner",
      },
    ];
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners,
      }),
    ).toBe("degraded");
  });

  it("returns online when all supplies and conditions are nominal", () => {
    const toners: TonerSupply[] = [
      {
        name: "Black Toner",
        color: "black",
        level: 80,
        max_capacity: 100,
        percent: 80,
        supply_type: "toner",
      },
    ];
    expect(
      getEffectivePrinterStatus({
        status: "online",
        toners,
        paperJam: false,
        outOfPaper: false,
      }),
    ).toBe("online");
  });
});
