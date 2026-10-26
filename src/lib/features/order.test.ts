import { describe, expect, it } from "vitest";
import {
  compareFeaturesAlphabetically,
  isDashboardFeature,
  orderDashboardWidgets,
  orderFeatures,
  orderSidebarItems,
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

  it("pins dashboard first and orders features alphabetically in sidebar", () => {
    const items = [
      { href: "/printers", label: "Printers" },
      { href: "/servers", label: "Servers" },
      { href: "/", label: "Dashboard" },
      { href: "/endpoint", label: "Endpoint" },
    ];

    const result = orderSidebarItems(items);
    expect(result.map((i) => i.label)).toEqual([
      "Dashboard",
      "Endpoint",
      "Printers",
      "Servers",
    ]);
    expect(result[0].href).toBe("/");
  });

  it("handles sidebar list without dashboard", () => {
    const items = [
      { href: "/servers", label: "Servers" },
      { href: "/endpoint", label: "Endpoint" },
    ];

    const result = orderSidebarItems(items);
    expect(result.map((i) => i.label)).toEqual(["Endpoint", "Servers"]);
  });

  it("excludes dashboard and sorts widgets alphabetically in dashboard", () => {
    const widgets = [
      { id: "printers", label: "Printers" },
      { id: "dashboard", label: "Dashboard" },
      { id: "servers", label: "Servers" },
      { id: "endpoint", label: "Endpoint" },
    ];

    const result = orderDashboardWidgets(widgets);
    expect(result.map((w) => w.label)).toEqual([
      "Endpoint",
      "Printers",
      "Servers",
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
});
