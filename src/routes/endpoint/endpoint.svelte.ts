import { invoke } from "@tauri-apps/api/core";
import type {
  EndpointMachineInfo,
  OsSupportInfo,
  ShellDetectionResult,
  TerminalCommandResult,
  TerminalEntry,
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
  terminalWorkingDir = $state<string>("~");
  terminalEntries = $state<TerminalEntry[]>([]);
  isCommandRunning = $state(false);

  private autoRefreshTimer: ReturnType<typeof setInterval> | null = null;
  private isDestroyed = false;

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

    await Promise.allSettled([
      this.fetchMachineInfo(),
      this.detectShell(),
      this.checkForUpdates(),
    ]);

    this.startAutoRefresh();
  }

  destroy(): void {
    this.isDestroyed = true;
    this.stopAutoRefresh();
  }

  startAutoRefresh(intervalMs = 3000): void {
    this.stopAutoRefresh();
    this.autoRefreshTimer = setInterval(() => {
      if (!this.isDestroyed && this.osSupport?.is_windows && !this.machineLoading) {
        this.fetchMachineInfo(false).catch((err) => {
          console.warn("Endpoint metrics refresh failed:", err);
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
      if (showSpinner) {
        this.machineLoading = false;
      }
    }
  }

  async checkForUpdates(): Promise<void> {
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
    this.operationStatus = "Downloading updates...";
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
    this.operationStatus = "Installing updates...";
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

  async executeCommand(rawCommand: string): Promise<void> {
    const trimmed = rawCommand.trim();
    if (!trimmed || this.isCommandRunning) return;

    if (trimmed === "cls" || trimmed === "clear") {
      this.clearTerminal();
      return;
    }

    const entryId = crypto.randomUUID();
    const workingDir = this.terminalWorkingDir;

    const entry: TerminalEntry = {
      id: entryId,
      command: trimmed,
      working_dir: workingDir,
      timestamp: new Date().toLocaleTimeString(),
      is_running: true,
    };

    this.terminalEntries.push(entry);
    this.isCommandRunning = true;

    try {
      const result = await invoke<TerminalCommandResult>("endpoint_execute_command", {
        command: trimmed,
        workingDir: workingDir === "~" ? null : workingDir,
      });

      entry.result = result;
      entry.is_running = false;
      if (result.working_dir) {
        this.terminalWorkingDir = result.working_dir;
      }
    } catch (err: unknown) {
      entry.is_running = false;
      entry.result = {
        command: trimmed,
        exit_code: 1,
        stdout: "",
        stderr: String(err),
        working_dir: workingDir,
      };
      throw err;
    } finally {
      this.isCommandRunning = false;
    }
  }

  clearTerminal(): void {
    this.terminalEntries = [];
  }
}

export const endpointStore = new EndpointStore();
