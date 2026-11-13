<script lang="ts">
  import StatusDot from "$lib/components/status-dot.svelte";
  import type { UpdateCheckResult } from "../types";

  interface Props {
    updates?: UpdateCheckResult | null;
    isRebootRequired: boolean;
    hasPendingDownloads: boolean;
    hasReadyInstalls: boolean;
    isBusy: boolean;
    downloadingId?: string | null;
    installingId?: string | null;
    onRestart: () => void;
    onDownloadAndInstall: () => void;
    onDownload: () => void;
    onInstall: () => void;
  }

  let {
    updates,
    isRebootRequired,
    hasPendingDownloads,
    hasReadyInstalls,
    isBusy,
    downloadingId,
    installingId,
    onRestart,
    onDownloadAndInstall,
    onDownload,
    onInstall,
  }: Props = $props();
</script>

<header class="page-header">
  <div class="title-cluster">
    <h1 class="page-title">System Updates</h1>

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
        <StatusDot status="warning" size={6} /> Restart Pending
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
  </div>
</header>

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
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

  .count-badge,
  .reboot-status-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    font-weight: 600;
    line-height: 1.2;
    padding: 0.18rem 0.55rem;
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
    background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px var(--color-warning);
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
  .btn-action-restart {
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

  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
