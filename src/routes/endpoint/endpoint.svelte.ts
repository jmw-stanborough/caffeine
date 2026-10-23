import { invoke } from "@tauri-apps/api/core";
import type {
  EndpointMachineInfo,
  OsSupportInfo,
  ShellDetectionResult,
  UpdateCheckResult,
  UpdateOperationResult,
} from "./types";

export class EndpointStore {
  osSupport = $state<OsSupportInfo | null>(null);
  osChecking = $state(true);

  machine = $state<EndpointMachineInfo | null>(null);
  machineLoading = $state(true);
  machineError = $state<string | null>(null);

  updates = $state<UpdateCheckResult | null>(null);
  updatesChecking = $state(false);
  updatesError = $state<string | null>(null);
  downloadingId = $state<string | null>(null);
  installingId = $state<string | null>(null);
  operationStatus = $state<string | null>(null);

  shell = $state<ShellDetectionResult | null>(null);

  private autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  private isDestroyed = false;
  private isRefreshing = false;

  async init(): Promise<void> {
    this.isDestroyed = false;
    this.osChecking = true;

    try {
      this.osSupport = await invoke<OsSupportInfo>("endpoint_check_os_support");
    } catch (err: unknown) {
      this.osSupport = {
        is_windows: false,
        os_name: "Unknown",
        os_version: "",
        architecture: "",
        hostname: "",
        error_message: String(err),
      };
    } finally {
      this.osChecking = false;
    }

    if (!this.osSupport?.is_windows) {
      return;
    }

    // Load initial hardware metrics and detect shell asynchronously
    await Promise.allSettled([
      this.fetchMachineInfo(true),
      this.detectShell(),
    ]);

    this.startAutoRefresh();
  }

  destroy(): void {
    this.isDestroyed = true;
    this.stopAutoRefresh();
  }

  startAutoRefresh(intervalMs = 3500): void {
    this.stopAutoRefresh();
    this.autoRefreshTimer = setInterval(() => {
      if (!this.isDestroyed && this.osSupport?.is_windows && !this.machineLoading && !this.isRefreshing) {
        this.fetchMachineInfo(false).catch((err) => {
          console.warn("Endpoint telemetry refresh failed:", err);
        });
      }
    }, intervalMs);
  }

  stopAutoRefresh(): void {
    if (this.autoRefreshTimer) {
      clearInterval(this.autoRefreshTimer);
      this.autoRefreshTimer = null;
    }
  }

  async fetchMachineInfo(showSpinner = true): Promise<void> {
    if (this.isRefreshing) return;
    this.isRefreshing = true;

    if (showSpinner) {
      this.machineLoading = true;
    }
    this.machineError = null;

    try {
      this.machine = await invoke<EndpointMachineInfo>("endpoint_get_machine_info");
    } catch (err: unknown) {
      this.machineError = String(err);
      throw err;
    } finally {
      this.isRefreshing = false;
      if (showSpinner) {
        this.machineLoading = false;
      }
    }
  }

  async checkForUpdates(): Promise<void> {
    if (this.updatesChecking) return;
    this.updatesChecking = true;
    this.updatesError = null;
    this.operationStatus = null;

    try {
      this.updates = await invoke<UpdateCheckResult>("endpoint_check_updates");
    } catch (err: unknown) {
      this.updatesError = String(err);
      throw err;
    } finally {
      this.updatesChecking = false;
    }
  }

  async downloadUpdates(targetId?: string): Promise<UpdateOperationResult> {
    this.downloadingId = targetId ?? "ALL";
    this.operationStatus = "Downloading updates in background...";
    this.updatesError = null;

    try {
      const res = await invoke<UpdateOperationResult>("endpoint_download_updates", {
        targetId: targetId ?? null,
      });
      this.operationStatus = res.message;
      await this.checkForUpdates();
      return res;
    } catch (err: unknown) {
      this.updatesError = String(err);
      this.operationStatus = `Download failed: ${err}`;
      throw err;
    } finally {
      this.downloadingId = null;
    }
  }

  async installUpdates(targetId?: string): Promise<UpdateOperationResult> {
    this.installingId = targetId ?? "ALL";
    this.operationStatus = "Installing updates in background...";
    this.updatesError = null;

    try {
      const res = await invoke<UpdateOperationResult>("endpoint_install_updates", {
        targetId: targetId ?? null,
      });
      this.operationStatus = res.message;
      await this.checkForUpdates();
      return res;
    } catch (err: unknown) {
      this.updatesError = String(err);
      this.operationStatus = `Installation failed: ${err}`;
      throw err;
    } finally {
      this.installingId = null;
    }
  }

  async detectShell(): Promise<void> {
    try {
      this.shell = await invoke<ShellDetectionResult>("endpoint_detect_shell");
    } catch (err: unknown) {
      console.warn("Failed to detect shell:", err);
      this.shell = {
        shell_name: "powershell",
        executable_path: "powershell",
        version: "5.1",
        is_pwsh: false,
      };
    }
  }
}

export const endpointStore = new EndpointStore();
