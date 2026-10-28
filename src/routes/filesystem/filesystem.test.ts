import { describe, expect, it } from "vitest";
import { FilesystemStore } from "./filesystem.svelte";
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

    // Mock navigateTo to avoid Tauri IPC in node unit test
    store.navigateTo = async (p: string) => {
      store.currentPath = p;
    };

    await store.selectVolume(vol);

    expect(store.selectedVolume).toBe(vol);
    expect(store.searchResults).toBeNull();
    expect(store.selectedPaths.size).toBe(0);
    expect(store.currentPath).toBe("/");
  });
});
