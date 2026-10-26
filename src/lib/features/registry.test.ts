import type { Component } from "svelte";
import { describe, expect, it } from "vitest";
import {
  getDashboardWidgets,
  getSidebarNavigationItems,
  registerFeature,
} from "./registry.svelte";

describe("feature registry", () => {
  it("provides sidebar items with Dashboard first followed by alphabetically sorted features", () => {
    const items = getSidebarNavigationItems();
    expect(items[0].label).toBe("Dashboard");
    expect(items[0].href).toBe("/");

    const featureLabels = items.slice(1).map((item) => item.label);
    const sortedFeatureLabels = [...featureLabels].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: "base", numeric: true }),
    );

    expect(featureLabels).toEqual(sortedFeatureLabels);
    expect(featureLabels).toEqual(["Endpoint", "Printers", "Servers"]);
  });

  it("provides dashboard widgets alphabetically sorted excluding dashboard", () => {
    const widgets = getDashboardWidgets();
    const widgetLabels = widgets.map((w) => w.label);

    expect(widgetLabels.includes("Dashboard")).toBe(false);

    const sortedLabels = [...widgetLabels].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: "base", numeric: true }),
    );

    expect(widgetLabels).toEqual(sortedLabels);
    expect(widgetLabels).toEqual(["Endpoint", "Printers", "Servers"]);
  });

  it("automatically sorts newly registered features in both sidebar and dashboard", () => {
    const DummyComponent = (() => null) as unknown as Component;
    registerFeature({
      id: "analytics",
      label: "Analytics",
      href: "/analytics",
      icon: DummyComponent,
      widget: DummyComponent,
    });

    const items = getSidebarNavigationItems();
    expect(items[0].label).toBe("Dashboard");
    expect(items[1].label).toBe("Analytics");

    const widgets = getDashboardWidgets();
    expect(widgets[0].label).toBe("Analytics");
  });
});
