import { beforeEach, describe, expect, it, vi } from "vitest";
import { EndpointStore } from "./endpoint.svelte";
import type { OsSupportInfo, UpdateOperationResult } from "./types";

const mockInvoke = vi.fn();
const mockListen = vi.fn().mockResolvedValue(() => {});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
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

  it("does not re-invoke OS checks or detectShell when osSupport is already populated", async () => {
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
      return null;
    });

    await store.init();
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_check_os_support");
    expect(store.osChecking).toBe(false);

    mockInvoke.mockClear();
    await store.init();
    expect(mockInvoke).not.toHaveBeenCalledWith("endpoint_check_os_support");
    expect(store.osChecking).toBe(false);
  });

  it("deduplicates concurrent init calls into a single invocation", async () => {
    const mockSupport: OsSupportInfo = {
      is_windows: true,
      os_name: "Windows 11 Pro",
      os_version: "10.0.26100",
      architecture: "x86_64",
      hostname: "DESKTOP-TEST",
      is_admin: true,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_check_os_support") {
        await new Promise((resolve) => setTimeout(resolve, 10));
        return mockSupport;
      }
      return null;
    });

    await Promise.all([store.init(), store.init(), store.init()]);
    const checkCalls = mockInvoke.mock.calls.filter(
      (call) => call[0] === "endpoint_check_os_support",
    );
    expect(checkCalls).toHaveLength(1);
    expect(store.osChecking).toBe(false);
  });

  it("invokes shell detection during init even on non-windows operating systems", async () => {
    const mockUnixSupport: OsSupportInfo = {
      is_windows: false,
      os_name: "Ubuntu",
      os_version: "24.04",
      architecture: "x86_64",
      hostname: "linux-dev",
      is_admin: false,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_check_os_support") return mockUnixSupport;
      if (cmd === "endpoint_detect_shell") {
        return {
          shell_name: "bash",
          executable_path: "/bin/bash",
          version: "GNU bash, version 5.2.21",
          is_pwsh: false,
        };
      }
      return null;
    });

    await store.init();
    expect(store.osSupport?.is_windows).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_detect_shell");
    expect(store.shell?.shell_name).toBe("bash");
  });

  it("blocks installUpdates immediately when reboot_required is true", async () => {
    store.updates = {
      updates: [
        {
          id: "upd-1",
          title: "KB1001",
          categories: ["Security"],
          is_mandatory: false,
          is_downloaded: true,
        },
      ],
      reboot_required: true,
      last_checked: "Just now",
    };

    const res = await store.installUpdates("upd-1");
    expect(res.success).toBe(false);
    expect(res.reboot_required).toBe(true);
    expect(store.updatesError).toContain("restart is required");
    expect(mockInvoke).not.toHaveBeenCalledWith("endpoint_install_updates", expect.anything());
  });

  it("supports batch download updates with multiple IDs", async () => {
    const successOp: UpdateOperationResult = {
      success: true,
      reboot_required: false,
      message: "Downloaded 2 updates.",
      succeeded_count: 2,
      failed_count: 0,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_download_updates") return successOp;
      if (cmd === "endpoint_check_updates") {
        return { updates: [], reboot_required: false, last_checked: "Just now" };
      }
      return null;
    });

    const res = await store.downloadUpdates(["id-1", "id-2"]);
    expect(res.success).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_download_updates", {
      targetIds: ["id-1", "id-2"],
      targetId: null,
    });
  });

  it("coordinates downloadAndInstallUpdates pipeline", async () => {
    const downloadOp: UpdateOperationResult = {
      success: true,
      reboot_required: false,
      message: "Download succeeded",
      succeeded_count: 1,
      failed_count: 0,
    };
    const installOp: UpdateOperationResult = {
      success: true,
      reboot_required: false,
      message: "Install succeeded",
      succeeded_count: 1,
      failed_count: 0,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_download_updates") return downloadOp;
      if (cmd === "endpoint_install_updates") return installOp;
      if (cmd === "endpoint_check_updates") {
        return { updates: [], reboot_required: false, last_checked: "Just now" };
      }
      return null;
    });

    const res = await store.downloadAndInstallUpdates("id-combo");
    expect(res.success).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_download_updates", {
      targetIds: ["id-combo"],
      targetId: "id-combo",
    });
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_install_updates", {
      targetIds: ["id-combo"],
      targetId: "id-combo",
    });
  });

  it("queues operations and processes them sequentially without blocking", async () => {
    const successOp: UpdateOperationResult = {
      success: true,
      reboot_required: false,
      message: "Done",
      succeeded_count: 1,
      failed_count: 0,
    };

    let downloadCalls = 0;
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_download_updates") {
        downloadCalls++;
        await new Promise((r) => setTimeout(r, 20));
        return successOp;
      }
      if (cmd === "endpoint_check_updates") {
        return { updates: [], reboot_required: false, last_checked: "Just now" };
      }
      return null;
    });

    store.queueOperation({ id: "item-1", title: "Item 1", type: "download" });
    store.queueOperation({ id: "item-2", title: "Item 2", type: "download" });

    expect(store.isUpdateActive("item-1")).toBe(true);
    expect(store.isUpdateQueued("item-2")).toBe(true);

    await new Promise((r) => setTimeout(r, 70));
    expect(downloadCalls).toBe(2);
    expect(store.operationQueue).toHaveLength(0);
  });

  it("invokes endpoint_restart_system on restartSystem", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_restart_system") return null;
      return null;
    });

    await store.restartSystem();
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_restart_system");
  });

  it("tracks update progress events and resolves progress for target update", async () => {
    let capturedCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation(async (eventName: string, cb: (event: { payload: unknown }) => void) => {
      if (eventName === "endpoint:update-progress") {
        capturedCallback = cb;
      }
      return () => {};
    });

    await store.setupEventListeners();
    expect(mockListen).toHaveBeenCalledWith("endpoint:update-progress", expect.any(Function));

    capturedCallback!({
      payload: {
        phase: "download",
        percent_complete: 42,
        current_index: 0,
        total_count: 2,
        bytes_downloaded: 1048576,
        total_bytes: 2097152,
        current_update_id: "update-123",
      },
    });

    expect(store.updateProgress).toEqual({
      phase: "download",
      percent_complete: 42,
      current_index: 0,
      total_count: 2,
      bytes_downloaded: 1048576,
      total_bytes: 2097152,
      current_update_id: "update-123",
    });

    expect(store.getUpdateProgress("update-123")).not.toBeNull();
    expect(store.getUpdateProgress("update-999")).toBeNull();
  });

  it("evaluates isOsSupported across Windows, Linux, macOS, and unsupported OS", async () => {
    expect(store.isOsSupported).toBe(false);

    store.osSupport = {
      is_windows: true,
      os_name: "Windows 11 Pro",
      os_version: "10.0.26100",
      architecture: "x86_64",
      hostname: "WIN-HOST",
      is_admin: true,
    };
    expect(store.isOsSupported).toBe(true);

    store.osSupport = {
      is_windows: false,
      is_supported: true,
      os_family: "linux",
      os_name: "Ubuntu",
      os_version: "24.04",
      architecture: "x86_64",
      hostname: "linux-host",
      is_admin: false,
    };
    expect(store.isOsSupported).toBe(true);

    store.osSupport = {
      is_windows: false,
      is_supported: true,
      os_family: "macos",
      os_name: "macOS",
      os_version: "15.1",
      architecture: "aarch64",
      hostname: "mac-host",
      is_admin: false,
    };
    expect(store.isOsSupported).toBe(true);

    store.osSupport = {
      is_windows: false,
      is_supported: false,
      os_family: "freebsd",
      os_name: "FreeBSD",
      os_version: "14.1",
      architecture: "x86_64",
      hostname: "bsd-host",
      is_admin: false,
    };
    expect(store.isOsSupported).toBe(false);
  });

  it("fetches machine info on Linux and macOS when is_supported is true", async () => {
    const linuxSupport: OsSupportInfo = {
      is_windows: false,
      is_supported: true,
      os_family: "linux",
      os_name: "Ubuntu",
      os_version: "24.04",
      architecture: "x86_64",
      hostname: "linux-host",
      is_admin: false,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "endpoint_check_os_support") return linuxSupport;
      if (cmd === "endpoint_detect_shell") return null;
      if (cmd === "endpoint_get_machine_info") {
        return {
          system: {
            computer_name: "linux-host",
            domain_or_workgroup: "",
            os_name: "Ubuntu 24.04",
            os_version: "24.04",
            build_number: "",
            architecture: "x86_64",
            uptime_seconds: 120,
            boot_timestamp: 0,
            fast_startup_enabled: false,
          },
          cpu: {
            model_name: "AMD Ryzen",
            vendor_id: "AuthenticAMD",
            physical_cores: 8,
            logical_cores: 16,
            frequency_mhz: 3600,
            global_usage_percent: 12,
            core_usages: [12],
          },
          memory: {
            total_bytes: 16_000_000_000,
            used_bytes: 4_000_000_000,
            free_bytes: 12_000_000_000,
            available_bytes: 12_000_000_000,
            usage_percent: 25,
          },
          disks: [],
          gpus: [],
          temperatures: [],
          networks: [],
        };
      }
      return null;
    });

    await store.subscribe();
    expect(store.isOsSupported).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("endpoint_get_machine_info");
    expect(store.machine?.system.computer_name).toBe("linux-host");
    store.unsubscribe();
  });

  it("queues and shifts pending terminal commands in FIFO order", () => {
    expect(store.pendingTerminalCommands).toEqual([]);
    expect(store.shiftTerminalCommand()).toBeUndefined();

    store.queueTerminalCommand("   ");
    expect(store.pendingTerminalCommands).toEqual([]);

    store.queueTerminalCommand("echo first");
    store.queueTerminalCommand("echo second\r");
    expect(store.pendingTerminalCommands).toEqual(["echo first", "echo second\r"]);

    expect(store.shiftTerminalCommand()).toBe("echo first");
    expect(store.shiftTerminalCommand()).toBe("echo second\r");
    expect(store.shiftTerminalCommand()).toBeUndefined();
    expect(store.pendingTerminalCommands).toEqual([]);
  });
});
