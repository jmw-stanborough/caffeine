import type { Component } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  getDashboardWidgets,
  getPersistentViews,
  getSidebarNavigationItems,
  getSidebarSections,
  handleSnapshotSync,
  initializeBackgroundFeatures,
  isSectionCollapsed,
  registerFeature,
  registerSidebarSubheader,
  setSectionCollapsed,
  toggleSectionCollapsed,
} from "./registry.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

function createMemoryStorage(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  };
}

describe("feature registry", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

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
      "File System",
      "Hardware",
      "Notes",
      "Printers",
      "Scripts",
      "Servers",
      "Terminal",
      "Todo",
      "Updates",
    ]);
  });

  it("provides sidebar sections ordered alphabetically with alphabetically ordered children", () => {
    const sections = getSidebarSections();
    expect(sections).toHaveLength(3);

    expect(sections[0].id).toBe("endpoint");
    expect(sections[0].label).toBe("Endpoint");
    expect(sections[0].items.map((i) => i.label)).toEqual([
      "File System",
      "Hardware",
      "Terminal",
      "Updates",
    ]);

    expect(sections[1].id).toBe("network");
    expect(sections[1].label).toBe("Network");
    expect(sections[1].items.map((i) => i.label)).toEqual(["Printers", "Servers"]);

    expect(sections[2].id).toBe("workspace");
    expect(sections[2].label).toBe("Workspace");
    expect(sections[2].items.map((i) => i.label)).toEqual([
      "Notes",
      "Scripts",
      "Todo",
    ]);
  });

  it("provides dashboard widgets alphabetically sorted excluding dashboard", () => {
    const widgets = getDashboardWidgets();
    const widgetLabels = widgets.map((w) => w.label);

    expect(widgetLabels.includes("Dashboard")).toBe(false);

    const sortedLabels = [...widgetLabels].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: "base", numeric: true }),
    );

    expect(widgetLabels).toEqual(sortedLabels);
    expect(widgetLabels).toEqual(["Hardware", "Printers", "Servers", "Workspace"]);
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

  it("toggles and persists sidebar section collapse state", async () => {
    const storage = createMemoryStorage();
    vi.stubGlobal("window", { localStorage: storage });

    setSectionCollapsed("workspace", false);
    expect(isSectionCollapsed("workspace")).toBe(false);

    toggleSectionCollapsed("workspace");
    expect(isSectionCollapsed("workspace")).toBe(true);
    expect(JSON.parse(storage.getItem("caffeine.sidebar.collapsed") ?? "{}")).toEqual({
      workspace: true,
    });

    toggleSectionCollapsed("workspace");
    expect(isSectionCollapsed("workspace")).toBe(false);
    expect(JSON.parse(storage.getItem("caffeine.sidebar.collapsed") ?? "{}")).toEqual({});

    setSectionCollapsed("network", true);
    vi.resetModules();
    const freshRegistry = await import("./registry.svelte");
    expect(freshRegistry.isSectionCollapsed("network")).toBe(true);
    expect(freshRegistry.isSectionCollapsed("endpoint")).toBe(false);
  });

  it("aggregates positive item badge counts onto each sidebar section", () => {
    const DummyComponent = (() => null) as unknown as Component;
    registerSidebarSubheader("ops", {
      label: "Operations",
      icon: DummyComponent,
    });
    registerFeature({
      id: "ops-alerts",
      label: "Ops Alerts",
      href: "/ops-alerts",
      icon: DummyComponent,
      group: "ops",
      badgeCount: () => 3,
    });
    registerFeature({
      id: "ops-audit",
      label: "Ops Audit",
      href: "/ops-audit",
      icon: DummyComponent,
      group: "ops",
      badgeCount: () => -2,
    });
    registerFeature({
      id: "ops-health",
      label: "Ops Health",
      href: "/ops-health",
      icon: DummyComponent,
      group: "ops",
      badgeCount: () => 5,
    });

    const sections = getSidebarSections();
    const opsSection = sections.find((s) => s.id === "ops");
    expect(opsSection?.badgeCount).toBe(8);
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

  it("returns persistent views for background loadable features with components", () => {
    const views = getPersistentViews();
    expect(views.some((v) => v.id === "terminal" && v.href === "/terminal")).toBe(true);
  });

  it("initializes background features and runs snapshot sync handlers", async () => {
    const initSpy = vi.fn().mockResolvedValue(undefined);
    const syncSpy = vi.fn().mockResolvedValue(undefined);
    const DummyComponent = (() => null) as unknown as Component;

    registerFeature({
      id: "test-bg-module",
      label: "Test BG Module",
      href: "/test-bg",
      icon: DummyComponent,
      backgroundLoadable: true,
      init: initSpy,
      onSnapshotSync: syncSpy,
    });

    await initializeBackgroundFeatures();
    expect(initSpy).toHaveBeenCalled();

    await handleSnapshotSync();
    expect(syncSpy).toHaveBeenCalled();
  });
});
