import { invoke } from "@tauri-apps/api/core";
import type {
  EndpointMachineInfo,
  OsSupportInfo,
  QueuedUpdateItem,
  ShellDetectionResult,
  UpdateCheckResult,
  UpdateOperationResult,
  UpdateOperationType,
} from "./types";

function normalizeTargetIds(target?: string | string[]): {
  single: string | null;
  list: string[] | null;
} {
  if (!target) {
    return { single: null, list: null };
  }
  if (Array.isArray(target)) {
    return {
      single: target.length === 1 ? target[0] : null,
      list: target,
    };
  }
  return { single: target, list: [target] };
}

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
  activeUpdateIds = $state<string[]>([]);
  operationStatus = $state<string | null>(null);
  operationQueue = $state<QueuedUpdateItem[]>([]);

  shell = $state<ShellDetectionResult | null>(null);

  private autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  private isRefreshing = false;
  private subscriberCount = 0;
  private initPromise: Promise<void> | null = null;

  get isBusy(): boolean {
    return this.updatesChecking || this.downloadingId !== null || this.installingId !== null;
  }

  isUpdateDownloading(id: string): boolean {
    if (this.downloadingId === id) return true;
    if (this.downloadingId !== null && this.activeUpdateIds.includes(id)) return true;
    return false;
  }

  isUpdateInstalling(id: string): boolean {
    if (this.installingId === id) return true;
    if (this.installingId !== null && this.activeUpdateIds.includes(id)) return true;
    return false;
  }

  isUpdateActive(id: string): boolean {
    return this.isUpdateDownloading(id) || this.isUpdateInstalling(id);
  }

  isUpdateQueued(id: string): boolean {
    return this.operationQueue.some((item) => item.id === id);
  }

  getQueuedOperation(id: string): QueuedUpdateItem | undefined {
    return this.operationQueue.find((item) => item.id === id);
  }

  async init(): Promise<void> {
    if (this.osSupport) {
      return;
    }
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = (async () => {
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

      await this.detectShell().catch((err) => {
        console.warn("Shell detection failed:", err);
      });

      if (!this.osSupport?.is_windows) {
        return;
      }

      if (this.subscriberCount > 0) {
        await this.fetchMachineInfo(true);
        this.startAutoRefresh();
      }
    })();

    try {
      await this.initPromise;
    } finally {
      this.initPromise = null;
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

  queueOperation(item: QueuedUpdateItem): void {
    if (this.isUpdateActive(item.id) || this.isUpdateQueued(item.id)) {
      return;
    }
    this.operationQueue = [...this.operationQueue, item];
    if (!this.isBusy) {
      void this.processNextQueueItem();
    }
  }

  cancelQueuedOperation(id: string): void {
    this.operationQueue = this.operationQueue.filter((item) => item.id !== id);
  }

  clearQueue(): void {
    this.operationQueue = [];
  }

  private async processNextQueueItem(): Promise<void> {
    if (this.isBusy || this.operationQueue.length === 0) {
      return;
    }
    const [nextItem, ...remaining] = this.operationQueue;
    this.operationQueue = remaining;

    try {
      if (nextItem.type === "download") {
        await this.downloadUpdates(nextItem.id);
      } else if (nextItem.type === "install") {
        await this.installUpdates(nextItem.id);
      } else if (nextItem.type === "download_and_install") {
        await this.downloadAndInstallUpdates(nextItem.id);
      }
    } catch (err: unknown) {
      console.warn(`Queued operation failed for ${nextItem.id}:`, err);
    } finally {
      if (this.operationQueue.length > 0 && !this.isBusy) {
        void this.processNextQueueItem();
      }
    }
  }

  async downloadUpdates(target?: string | string[]): Promise<UpdateOperationResult> {
    const { single, list } = normalizeTargetIds(target);
    this.downloadingId = single ?? (list ? (list.length === 1 ? list[0] : "ALL") : "ALL");
    this.activeUpdateIds = list ?? this.updates?.updates.map((u) => u.id) ?? [];
    this.operationStatus = "Downloading updates in background...";
    this.updatesError = null;

    try {
      const res = await invoke<UpdateOperationResult>("endpoint_download_updates", {
        targetIds: list,
        targetId: single,
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
      this.activeUpdateIds = [];
      if (this.operationQueue.length > 0 && !this.isBusy) {
        void this.processNextQueueItem();
      }
    }
  }

  async installUpdates(target?: string | string[]): Promise<UpdateOperationResult> {
    if (this.updates?.reboot_required) {
      const rebootMsg = "A system restart is required before installing further updates.";
      this.updatesError = rebootMsg;
      this.operationStatus = null;
      return {
        success: false,
        reboot_required: true,
        message: rebootMsg,
        succeeded_count: 0,
        failed_count: 0,
      };
    }

    const { single, list } = normalizeTargetIds(target);
    this.installingId = single ?? (list ? (list.length === 1 ? list[0] : "ALL") : "ALL");
    this.activeUpdateIds = list ?? this.updates?.updates.map((u) => u.id) ?? [];
    this.operationStatus = "Installing updates in background...";
    this.updatesError = null;

    try {
      const res = await invoke<UpdateOperationResult>("endpoint_install_updates", {
        targetIds: list,
        targetId: single,
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
      this.activeUpdateIds = [];
      if (this.operationQueue.length > 0 && !this.isBusy) {
        void this.processNextQueueItem();
      }
    }
  }

  async downloadAndInstallUpdates(target?: string | string[]): Promise<UpdateOperationResult> {
    const downloadRes = await this.downloadUpdates(target);
    if (!downloadRes.success) {
      return downloadRes;
    }

    if (downloadRes.reboot_required || this.updates?.reboot_required) {
      this.operationStatus = "Updates downloaded. System restart is required before installation.";
      return downloadRes;
    }

    this.operationStatus = "Download complete. Installing updates...";
    return await this.installUpdates(target);
  }

  async restartSystem(): Promise<void> {
    this.operationStatus = "Initiating system restart...";
    this.updatesError = null;

    try {
      await invoke("endpoint_restart_system");
    } catch (err: unknown) {
      this.updatesError = `System restart failed: ${String(err)}`;
      this.operationStatus = null;
      throw err;
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
