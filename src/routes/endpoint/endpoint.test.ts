import { beforeEach, describe, expect, it, vi } from "vitest";
import { EndpointStore } from "./endpoint.svelte";
import type { OsSupportInfo, UpdateOperationResult } from "./types";

const mockInvoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("EndpointStore", () => {
  let store: EndpointStore;

  beforeEach(() => {
    vi.clearAllMocks();
    store = new EndpointStore();
  });

  it("evaluates isAdmin as false initially or when osSupport is null", () => {
    expect(store.osSupport).toBeNull();
    expect(store.isAdmin).toBe(false);
  });

  it("reflects is_admin status when osSupport is populated", async () => {
    const mockSupport: OsSupportInfo = {
      is_windows: true,
      os_name: "Windows 11 Pro",
      os_version: "10.0.26100",
      architecture: "x86_64",
      hostname: "DESKTOP-TEST",
      is_admin: true,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_check_os_support") return mockSupport;
      if (cmd === "endpoint_get_machine_info") return null;
      if (cmd === "endpoint_detect_shell") return null;
      return null;
    });

    await store.init();
    expect(store.osSupport?.is_admin).toBe(true);
    expect(store.isAdmin).toBe(true);

    store.osSupport = { ...mockSupport, is_admin: false };
    expect(store.isAdmin).toBe(false);
  });

  it("captures and retains updatesError when downloadUpdates returns success: false", async () => {
    const failedOp: UpdateOperationResult = {
      success: false,
      reboot_required: false,
      message: "Download failed (code 4). Administrative privileges or a system restart may be required.",
      succeeded_count: 0,
      failed_count: 1,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_download_updates") return failedOp;
      return null;
    });

    const res = await store.downloadUpdates("update-123");
    expect(res.success).toBe(false);
    expect(store.updatesError).toBe(
      "Download failed (code 4). Administrative privileges or a system restart may be required.",
    );
    expect(store.operationStatus).toBeNull();
    expect(store.downloadingId).toBeNull();
  });

  it("captures and retains updatesError when installUpdates returns success: false", async () => {
    const failedOp: UpdateOperationResult = {
      success: false,
      reboot_required: false,
      message: "Installation failed (code 4). Administrative privileges or a system restart may be required.",
      succeeded_count: 0,
      failed_count: 1,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_install_updates") return failedOp;
      return null;
    });

    const res = await store.installUpdates("update-123");
    expect(res.success).toBe(false);
    expect(store.updatesError).toBe(
      "Installation failed (code 4). Administrative privileges or a system restart may be required.",
    );
    expect(store.operationStatus).toBeNull();
    expect(store.installingId).toBeNull();
  });
});
