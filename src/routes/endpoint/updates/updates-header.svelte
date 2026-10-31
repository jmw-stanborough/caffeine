<script lang="ts">
  import type { UpdateCheckResult } from "../types";

  interface Props {
    updates?: UpdateCheckResult | null;
    isRebootRequired: boolean;
    hasPendingDownloads: boolean;
    hasReadyInstalls: boolean;
    isBusy: boolean;
    downloadingId?: string | null;
    installingId?: string | null;
    isChecking: boolean;
    onRestart: () => void;
    onDownloadAndInstall: () => void;
    onDownload: () => void;
    onInstall: () => void;
    onCheck: () => void;
  }

  let {
    updates,
    isRebootRequired,
    hasPendingDownloads,
    hasReadyInstalls,
    isBusy,
    downloadingId,
    installingId,
    isChecking,
    onRestart,
    onDownloadAndInstall,
    onDownload,
    onInstall,
    onCheck,
  }: Props = $props();
</script>

<div class="window-header">
  <div class="title-cluster">
    <div class="header-icon-wrap">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
        <path d="M16 21h5v-5" />
      </svg>
    </div>
    <h3 class="window-title">System Updates</h3>

    {#if updates}
      <span
        class="count-badge"
        class:badge-has-updates={updates.updates.length > 0}
        class:badge-clean={updates.updates.length === 0}
      >
        {updates.updates.length} Available
      </span>
    {/if}

    {#if isRebootRequired}
      <span class="reboot-status-pill">
        <span class="pulse-dot"></span> Restart Pending
      </span>
    {/if}
  </div>

  <div class="header-actions">
    {#if isRebootRequired}
      <button
        class="btn-action-restart"
        onclick={onRestart}
        title="Restart the system immediately to complete updates"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
        </svg>
        <span>Restart Now</span>
      </button>
    {/if}

    {#if updates && updates.updates.length > 0}
      <button
        class="btn-action-primary"
        onclick={onDownloadAndInstall}
        disabled={isBusy || isRebootRequired}
        title={isRebootRequired
          ? "A system restart is required before updates can be installed."
          : "Download and install all available updates in one step"}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        <span>
          {downloadingId === "ALL" && installingId === "ALL"
            ? "Processing..."
            : "Download & Install All"}
        </span>
      </button>

      {#if hasPendingDownloads}
        <button
          class="btn-action-secondary"
          onclick={onDownload}
          disabled={isBusy}
          title="Download all pending update packages"
        >
          <span>{downloadingId === "ALL" ? "Downloading..." : "Download All"}</span>
        </button>
      {/if}

      {#if hasReadyInstalls}
        <button
          class="btn-action-accent"
          onclick={onInstall}
          disabled={isBusy || isRebootRequired}
          title={isRebootRequired
            ? "A system restart is required before installing updates."
            : "Install all downloaded updates"}
        >
          <span>{installingId === "ALL" ? "Installing..." : "Install All"}</span>
        </button>
      {/if}
    {/if}

    <button
      class="btn-window-action"
      onclick={onCheck}
      disabled={isBusy}
      title="Check for Windows updates"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class:spin={isChecking}
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <span>{isChecking ? "Checking..." : "Check"}</span>
    </button>
  </div>
</div>

<style>
  .window-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .title-cluster {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .header-icon-wrap {
    color: var(--color-primary);
    display: flex;
    align-items: center;
  }

  .window-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
  }

  .count-badge {
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.15rem 0.5rem;
    border-radius: 999px;
  }

  .count-badge.badge-has-updates {
    background-color: color-mix(in srgb, var(--color-primary) 20%, var(--color-surface));
    color: var(--color-primary);
    border: solid 1px var(--color-primary);
  }

  .count-badge.badge-clean {
    background-color: color-mix(in srgb, var(--color-success) 20%, var(--color-surface));
    color: var(--color-success);
    border: solid 1px var(--color-success);
  }

  .reboot-status-pill {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.15rem 0.55rem;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px var(--color-warning);
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-warning);
    animation: pulse 1.8s infinite ease-in-out;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(0.8); }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .btn-action-primary,
  .btn-action-secondary,
  .btn-action-accent,
  .btn-action-restart,
  .btn-window-action {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.32rem 0.65rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-action-primary {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-action-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-action-secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-action-secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .btn-action-accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-action-accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-action-restart {
    background-color: var(--color-warning);
    border: solid 1px var(--color-warning);
    color: var(--color-base);
  }

  .btn-action-restart:hover {
    filter: brightness(1.1);
  }

  .btn-window-action {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-window-action:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
