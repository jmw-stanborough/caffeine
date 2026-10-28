import type { Component } from "svelte";
import { describe, expect, it } from "vitest";
import {
  getDashboardWidgets,
  getSidebarNavigationItems,
  getSidebarSections,
  registerFeature,
  registerSidebarSubheader,
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
    expect(featureLabels).toEqual([
      "Hardware",
      "Printers",
      "Servers",
      "Terminal",
      "Updates",
    ]);
  });

  it("provides sidebar sections ordered alphabetically with alphabetically ordered children", () => {
    const sections = getSidebarSections();
    expect(sections).toHaveLength(2);

    expect(sections[0].id).toBe("endpoint");
    expect(sections[0].label).toBe("Endpoint");
    expect(sections[0].items.map((i) => i.label)).toEqual([
      "Hardware",
      "Terminal",
      "Updates",
    ]);

    expect(sections[1].id).toBe("network");
    expect(sections[1].label).toBe("Network");
    expect(sections[1].items.map((i) => i.label)).toEqual(["Printers", "Servers"]);
  });

  it("provides dashboard widgets alphabetically sorted excluding dashboard", () => {
    const widgets = getDashboardWidgets();
    const widgetLabels = widgets.map((w) => w.label);

    expect(widgetLabels.includes("Dashboard")).toBe(false);

    const sortedLabels = [...widgetLabels].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: "base", numeric: true }),
    );

    expect(widgetLabels).toEqual(sortedLabels);
    expect(widgetLabels).toEqual(["Hardware", "Printers", "Servers"]);
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

  it("automatically sorts newly registered subheaders and their children alphabetically", () => {
    const DummyComponent = (() => null) as unknown as Component;
    registerSidebarSubheader("compliance", {
      label: "Compliance",
      icon: DummyComponent,
    });
    registerFeature({
      id: "audits",
      label: "Audits",
      href: "/audits",
      icon: DummyComponent,
      group: "compliance",
    });
    registerFeature({
      id: "access",
      label: "Access Control",
      href: "/access",
      icon: DummyComponent,
      group: "compliance",
    });

    const sections = getSidebarSections();
    const complianceSection = sections.find((s) => s.id === "compliance");
    expect(complianceSection).toBeDefined();
    expect(complianceSection?.label).toBe("Compliance");
    expect(complianceSection?.items.map((i) => i.label)).toEqual([
      "Access Control",
      "Audits",
    ]);

    const sectionLabels = sections.map((s) => s.label);
    const sortedSectionLabels = [...sectionLabels].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: "base", numeric: true }),
    );
    expect(sectionLabels).toEqual(sortedSectionLabels);
  });

  it("throws descriptive error when feature has unregistered sidebar group", () => {
    const DummyComponent = (() => null) as unknown as Component;
    registerFeature({
      id: "orphan",
      label: "Orphan Feature",
      href: "/orphan",
      icon: DummyComponent,
      group: "nonexistent-group",
    });

    expect(() => getSidebarSections()).toThrowError(
      'Missing sidebar subheader definition for group "nonexistent-group" in feature "orphan"',
    );
  });
});
