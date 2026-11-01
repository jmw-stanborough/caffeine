import { describe, expect, it } from "vitest";
import type { SystemUpdateItem } from "../types";
import { filterAndSortUpdates } from "./filter";
import { getSeverityClass, getSeverityWeight } from "./severity";

describe("severity helpers", () => {
  it("computes severity classes accurately", () => {
    expect(getSeverityClass("Critical")).toBe("sev-critical");
    expect(getSeverityClass("Important")).toBe("sev-important");
    expect(getSeverityClass("Moderate")).toBe("sev-moderate");
    expect(getSeverityClass("Low")).toBe("sev-low");
    expect(getSeverityClass("Unknown")).toBe("sev-other");
    expect(getSeverityClass(undefined)).toBe("sev-none");
  });

  it("computes severity weights accurately", () => {
    expect(getSeverityWeight("Critical")).toBe(4);
    expect(getSeverityWeight("Important")).toBe(3);
    expect(getSeverityWeight("Moderate")).toBe(2);
    expect(getSeverityWeight("Low")).toBe(1);
    expect(getSeverityWeight(undefined)).toBe(0);
  });
});

describe("filterAndSortUpdates", () => {
  const sampleUpdates: SystemUpdateItem[] = [
    {
      id: "upd-1",
      title: "Security Update Alpha",
      kb_article_id: "KB1001",
      categories: ["Security"],
      severity: "Critical",
      is_mandatory: true,
      is_downloaded: false,
      size_bytes: 500,
    },
    {
      id: "upd-2",
      title: "Driver Update Beta",
      kb_article_id: "KB2002",
      categories: ["Drivers"],
      severity: "Low",
      is_mandatory: false,
      is_downloaded: true,
      size_bytes: 1200,
    },
    {
      id: "upd-3",
      title: "Feature Update Gamma",
      kb_article_id: "KB3003",
      categories: ["Features"],
      severity: "Important",
      is_mandatory: false,
      is_downloaded: false,
      size_bytes: 800,
    },
  ];

  it("returns empty array when updates list is undefined", () => {
    expect(filterAndSortUpdates(undefined, "all", "", "severity")).toEqual([]);
  });

  it("filters by pending_download", () => {
    const res = filterAndSortUpdates(sampleUpdates, "pending_download", "", "title");
    expect(res.map((u) => u.id)).toEqual(["upd-3", "upd-1"]);
  });

  it("filters by ready_to_install", () => {
    const res = filterAndSortUpdates(sampleUpdates, "ready_to_install", "", "title");
    expect(res.map((u) => u.id)).toEqual(["upd-2"]);
  });

  it("filters by mandatory", () => {
    const res = filterAndSortUpdates(sampleUpdates, "mandatory", "", "title");
    expect(res.map((u) => u.id)).toEqual(["upd-1"]);
  });

  it("filters by search query matching title or kb", () => {
    const res = filterAndSortUpdates(sampleUpdates, "all", "beta", "title");
    expect(res.map((u) => u.id)).toEqual(["upd-2"]);

    const kbRes = filterAndSortUpdates(sampleUpdates, "all", "KB1001", "title");
    expect(kbRes.map((u) => u.id)).toEqual(["upd-1"]);
  });

  it("sorts by severity descending", () => {
    const res = filterAndSortUpdates(sampleUpdates, "all", "", "severity");
    expect(res.map((u) => u.id)).toEqual(["upd-1", "upd-3", "upd-2"]);
  });

  it("sorts by size descending", () => {
    const res = filterAndSortUpdates(sampleUpdates, "all", "", "size");
    expect(res.map((u) => u.id)).toEqual(["upd-2", "upd-3", "upd-1"]);
  });
});
