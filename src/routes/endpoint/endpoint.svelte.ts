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
  isAdmin = $derived(this.osSupport?.is_admin ?? false);

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
  private isRefreshing = false;
  private subscriberCount = 0;

  async init(): Promise<void> {
    this.osChecking = true;

    try {
      this.osSupport = await invoke<OsSupportInfo>("endpoint_check_os_support");
    } catch (err: unknown) {
      this.osSupport = null;
      this.machineError = String(err);
      throw err;
    } finally {
      this.osChecking = false;
    }

    if (!this.osSupport?.is_windows) {
      return;
    }

    await Promise.allSettled([
      this.fetchMachineInfo(true),
      this.detectShell(),
    ]);

    if (this.subscriberCount > 0) {
      this.startAutoRefresh();
    }
  }

  async subscribe(): Promise<void> {
    this.subscriberCount++;
    if (!this.osSupport) {
      await this.init();
    } else {
      if (this.osSupport.is_windows && !this.machine) {
        await this.fetchMachineInfo(true);
      }
      this.startAutoRefresh();
    }
  }

  unsubscribe(): void {
    this.subscriberCount = Math.max(0, this.subscriberCount - 1);
    if (this.subscriberCount === 0) {
      this.stopAutoRefresh();
    }
  }

  startAutoRefresh(intervalMs = 3500): void {
    this.stopAutoRefresh();
    this.autoRefreshTimer = setInterval(() => {
      if (this.subscriberCount > 0 && this.osSupport?.is_windows && !this.isRefreshing) {
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

    if (showSpinner && !this.machine) {
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
      if (!res.success) {
        this.updatesError = res.message || "Failed to download updates.";
        this.operationStatus = null;
      } else {
        this.operationStatus = res.message;
        await this.checkForUpdates();
      }
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
      if (!res.success) {
        this.updatesError = res.message || "Failed to install updates.";
        this.operationStatus = null;
      } else {
        this.operationStatus = res.message;
        await this.checkForUpdates();
      }
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
      this.shell = null;
      throw new Error(`Failed to detect shell: ${String(err)}`);
    }
  }
}

export const endpointStore = new EndpointStore();
