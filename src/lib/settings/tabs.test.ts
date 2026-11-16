import { describe, expect, it, vi } from "vitest";
import { isRemoteHost } from "$lib/client-mode.svelte";
import { orderSettingsTabs, SETTINGS_TABS } from "./tabs";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("settings tabs", () => {
  it("orders registered settings tabs alphabetically", () => {
    expect(SETTINGS_TABS.map((tab) => tab.label)).toEqual([
      "Appearance",
      "Log Retention",
      "Network",
      "Sound Effects",
      "Workspace",
    ]);
  });

  it("sorts arbitrary tabs case-insensitively and naturally without mutating input", () => {
    const input = [
      { id: "ws", label: "workspace" },
      { id: "t10", label: "Tab 10" },
      { id: "app", label: "Appearance" },
      { id: "t2", label: "tab 2" },
    ];

    const sorted = orderSettingsTabs(input);

    expect(sorted.map((tab) => tab.label)).toEqual([
      "Appearance",
      "tab 2",
      "Tab 10",
      "workspace",
    ]);
    expect(input.map((tab) => tab.id)).toEqual(["ws", "t10", "app", "t2"]);
  });
});

describe("isRemoteHost", () => {
  it("treats empty and local loopback hosts as non-remote", () => {
    expect(isRemoteHost("")).toBe(false);
    expect(isRemoteHost("   ")).toBe(false);
    expect(isRemoteHost("127.0.0.1")).toBe(false);
    expect(isRemoteHost("localhost")).toBe(false);
    expect(isRemoteHost("  LOCALHOST  ")).toBe(false);
    expect(isRemoteHost("::1")).toBe(false);
    expect(isRemoteHost("0.0.0.0")).toBe(false);
  });

  it("identifies remote IPs and hostnames as remote", () => {
    expect(isRemoteHost("192.168.1.50")).toBe(true);
    expect(isRemoteHost("server.domain.lan")).toBe(true);
    expect(isRemoteHost("  10.0.0.5 ")).toBe(true);
  });
});
