import { describe, expect, it } from "vitest";
import { getEffectivePrinterStatus } from "./status";
import { filterPrintersByTags, getPrinterTags, isFailureTag } from "./tags";
import type { Printer, TonerSupply } from "./types";

const samplePrinter: Printer = {
  id: "p1",
  host: "192.168.1.50",
  label: "Main Office",
  snmp_community: "public",
};

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

describe("disconnected printer badge", () => {
  it("classifies Disconnected as a failure tag", () => {
    expect(isFailureTag("Disconnected")).toBe(true);
    expect(isFailureTag("disconnected")).toBe(true);
  });

  it("includes Disconnected tag when printer status is offline", () => {
    const tags = getPrinterTags(samplePrinter, {
      status: "offline",
      toners: [],
    });
    expect(tags).toContain("Disconnected");
  });

  it("omits Disconnected tag when printer is online or checking", () => {
    expect(
      getPrinterTags(samplePrinter, { status: "online", toners: [] }),
    ).not.toContain("Disconnected");
    expect(
      getPrinterTags(samplePrinter, { status: "checking", toners: [] }),
    ).not.toContain("Disconnected");
  });

  it("matches offline printers when filtering by Disconnected tag", () => {
    const onlinePrinter: Printer = { ...samplePrinter, id: "p2" };
    const matched = filterPrintersByTags(
      [samplePrinter, onlinePrinter],
      {
        p1: { status: "offline", toners: [] },
        p2: { status: "online", toners: [] },
      },
      ["Disconnected"],
    );
    expect(matched).toEqual([samplePrinter]);
  });
});
