import { describe, expect, it, vi } from "vitest";
import { FilesystemStore } from "./filesystem.svelte";
import { buildBreadcrumbs, getParentPath } from "./path";
import type { FileEntry, StorageVolume } from "./types";

describe("FilesystemStore", () => {
  it("computes totalSelectedBytes correctly for selected items", () => {
    const store = new FilesystemStore();
    const mockEntries: FileEntry[] = [
      {
        path: "/test/file1.txt",
        name: "file1.txt",
        is_dir: false,
        size_bytes: 1024,
        drive_usage_percent: 0.1,
      },
      {
        path: "/test/file2.txt",
        name: "file2.txt",
        is_dir: false,
        size_bytes: 2048,
        drive_usage_percent: 0.2,
      },
      {
        path: "/test/dir1",
        name: "dir1",
        is_dir: true,
        size_bytes: 4096,
        drive_usage_percent: 0.4,
      },
    ];

    store.listing = {
      path: "/test",
      entries: mockEntries,
      total_files: 2,
      total_dirs: 1,
      total_bytes: 7168,
    };

    store.toggleSelect("/test/file1.txt");
    store.toggleSelect("/test/dir1");

    expect(store.totalSelectedBytes).toBe(5120);

    store.selectAll();
    expect(store.totalSelectedBytes).toBe(7168);

    store.deselectAll();
    expect(store.totalSelectedBytes).toBe(0);
  });

  it("handles volume selection and clears prior search states", async () => {
    const store = new FilesystemStore();
    const vol: StorageVolume = {
      id: "vol-1",
      name: "Root",
      mount_point: "/",
      file_system: "ext4",
      total_bytes: 100000,
      used_bytes: 50000,
      available_bytes: 50000,
      usage_percent: 50.0,
      is_removable: false,
    };

    store.searchResults = [
      {
        path: "/old.txt",
        name: "old.txt",
        is_dir: false,
        size_bytes: 10,
        drive_usage_percent: 0.01,
      },
    ];

    store.navigateTo = async (p: string) => {
      store.currentPath = p;
    };

    await store.selectVolume(vol);

    expect(store.selectedVolume).toBe(vol);
    expect(store.searchResults).toBeNull();
    expect(store.selectedPaths.size).toBe(0);
    expect(store.currentPath).toBe("/");
  });

  it("navigates up through Windows and POSIX paths and stops at roots", async () => {
    const store = new FilesystemStore();
    const visited: string[] = [];
    store.navigateTo = vi.fn(async (p: string) => {
      visited.push(p);
      store.currentPath = p;
    });

    store.currentPath = "C:\\Users\\Admin";
    await store.navigateUp();
    expect(store.currentPath).toBe("C:\\Users");

    await store.navigateUp();
    expect(store.currentPath).toBe("C:\\");

    await store.navigateUp();
    expect(store.currentPath).toBe("C:\\");

    store.currentPath = "/var/log";
    await store.navigateUp();
    expect(store.currentPath).toBe("/var");

    await store.navigateUp();
    expect(store.currentPath).toBe("/");

    await store.navigateUp();
    expect(store.currentPath).toBe("/");

    expect(visited).toEqual(["C:\\Users", "C:\\", "/var", "/"]);
  });
});

describe("buildBreadcrumbs", () => {
  it("returns empty array for empty path", () => {
    expect(buildBreadcrumbs("")).toEqual([]);
  });

  it("builds Windows drive root and nested breadcrumbs", () => {
    expect(buildBreadcrumbs("C:\\")).toEqual([{ label: "C:", fullPath: "C:\\" }]);
    expect(buildBreadcrumbs("C:\\Users")).toEqual([
      { label: "C:", fullPath: "C:\\" },
      { label: "Users", fullPath: "C:\\Users" },
    ]);
    expect(buildBreadcrumbs("C:\\Users\\Admin")).toEqual([
      { label: "C:", fullPath: "C:\\" },
      { label: "Users", fullPath: "C:\\Users" },
      { label: "Admin", fullPath: "C:\\Users\\Admin" },
    ]);
  });

  it("builds POSIX root and nested breadcrumbs", () => {
    expect(buildBreadcrumbs("/")).toEqual([{ label: "/", fullPath: "/" }]);
    expect(buildBreadcrumbs("/var")).toEqual([
      { label: "/", fullPath: "/" },
      { label: "var", fullPath: "/var" },
    ]);
    expect(buildBreadcrumbs("/var/log")).toEqual([
      { label: "/", fullPath: "/" },
      { label: "var", fullPath: "/var" },
      { label: "log", fullPath: "/var/log" },
    ]);
  });
});

describe("getParentPath", () => {
  it("returns null for empty path", () => {
    expect(getParentPath("")).toBeNull();
  });

  it("handles Windows roots and nested paths", () => {
    expect(getParentPath("C:\\")).toBeNull();
    expect(getParentPath("C:")).toBeNull();
    expect(getParentPath("C:\\Users")).toBe("C:\\");
    expect(getParentPath("C:\\Users\\Admin")).toBe("C:\\Users");
    expect(getParentPath("C:\\Users\\Admin\\")).toBe("C:\\Users");
  });

  it("handles POSIX roots and nested paths", () => {
    expect(getParentPath("/")).toBeNull();
    expect(getParentPath("/var")).toBe("/");
    expect(getParentPath("/var/log")).toBe("/var");
    expect(getParentPath("/var/log/")).toBe("/var");
  });
});
