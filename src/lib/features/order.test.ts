import { describe, expect, it } from "vitest";
import {
  compareFeaturesAlphabetically,
  isDashboardFeature,
  orderFeatures,
  orderSidebarSections,
} from "./order";

describe("feature ordering", () => {
  it("detects root route and dashboard identifiers", () => {
    expect(isDashboardFeature({ href: "/" })).toBe(true);
    expect(isDashboardFeature({ id: "dashboard" })).toBe(true);
    expect(isDashboardFeature({ label: "Dashboard" })).toBe(true);
    expect(isDashboardFeature({ label: "  dashboard  " })).toBe(true);
    expect(isDashboardFeature({ href: "/printers", label: "Printers" })).toBe(false);
  });

  it("sorts ascending case-insensitively and naturally", () => {
    const items = [
      { label: "servers" },
      { label: "Endpoint" },
      { label: "Printers 10" },
      { label: "printers 2" },
    ];

    const sorted = [...items].sort(compareFeaturesAlphabetically);
    expect(sorted.map((i) => i.label)).toEqual([
      "Endpoint",
      "printers 2",
      "Printers 10",
      "servers",
    ]);
  });

  it("excludes dashboard and sorts features alphabetically", () => {
    const features = [
      { id: "servers", label: "Servers" },
      { id: "dashboard", label: "Dashboard" },
      { id: "printers", label: "Printers" },
      { id: "endpoint", label: "Endpoint" },
    ];

    const result = orderFeatures(features);
    expect(result.map((f) => f.label)).toEqual([
      "Endpoint",
      "Printers",
      "Servers",
    ]);
  });

  it("orders sidebar sections alphabetically and orders each section's children alphabetically", () => {
    const sections = [
      {
        id: "network",
        label: "Network",
        items: [
          { label: "Servers" },
          { label: "Printers" },
        ],
      },
      {
        id: "endpoint",
        label: "Endpoint",
        items: [
          { label: "Updates" },
          { label: "Hardware" },
          { label: "Terminal" },
        ],
      },
    ];

    const result = orderSidebarSections(sections);

    expect(result.map((s) => s.label)).toEqual(["Endpoint", "Network"]);
    expect(result[0].items.map((i) => i.label)).toEqual([
      "Hardware",
      "Terminal",
      "Updates",
    ]);
    expect(result[1].items.map((i) => i.label)).toEqual(["Printers", "Servers"]);
  });

  it("handles sections with empty child items without mutating inputs", () => {
    const originalItems = [{ label: "Beta" }, { label: "Alpha" }];
    const originalSections = [
      { id: "b", label: "Beta Section", items: originalItems },
      { id: "a", label: "Alpha Section", items: [] },
    ];

    const result = orderSidebarSections(originalSections);

    expect(result[0].id).toBe("a");
    expect(result[0].items).toEqual([]);
    expect(result[1].id).toBe("b");
    expect(result[1].items.map((i) => i.label)).toEqual(["Alpha", "Beta"]);
    expect(originalItems[0].label).toBe("Beta");
    expect(originalSections[0].id).toBe("b");
  });
});
