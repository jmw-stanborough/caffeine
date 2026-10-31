<script lang="ts">
  import { formatBytes } from "../metrics";
  import type { UpdateProgressEvent } from "../types";

  interface Props {
    updateId: string;
    isDownloaded: boolean;
    isDownloading: boolean;
    isInstalling: boolean;
    progress?: UpdateProgressEvent | null;
    isQueued: boolean;
    rebootRequired: boolean;
    onDownload: (id: string) => void;
    onInstall: (id: string) => void;
    onDownloadAndInstall: (id: string) => void;
    onCancelQueue: (id: string) => void;
  }

  let {
    updateId,
    isDownloaded,
    isDownloading,
    isInstalling,
    progress,
    isQueued,
    rebootRequired,
    onDownload,
    onInstall,
    onDownloadAndInstall,
    onCancelQueue,
  }: Props = $props();
</script>

<div class="update-card-actions">
  {#if isDownloading || isInstalling}
    <div class="card-progress-indicator">
      <div class="card-progress-header">
        <div class="card-progress-label-group">
          <span class="mini-spinner"></span>
          <span class="card-progress-label">
            {isDownloading ? "Downloading..." : "Installing..."}
          </span>
        </div>
        <span class="card-progress-percent">
          {progress ? `${progress.percent_complete}%` : ""}
        </span>
      </div>
      <div
        class="card-progress-track"
        role="progressbar"
        aria-valuenow={progress?.percent_complete ?? 0}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          class="card-progress-fill"
          class:indeterminate={!progress || progress.percent_complete === 0}
          style:width="{progress ? Math.max(progress.percent_complete, 5) : 100}%"
        ></div>
      </div>
      {#if progress && isDownloading && progress.bytes_downloaded != null && progress.total_bytes}
        <div class="card-progress-bytes">
          {formatBytes(progress.bytes_downloaded)} / {formatBytes(progress.total_bytes)}
        </div>
      {/if}
    </div>
  {:else if isQueued}
    <button
      class="btn-card-action cancel-queue"
      onclick={() => onCancelQueue(updateId)}
      title="Remove this update from the queue"
    >
      Cancel Queued
    </button>
  {:else if !isDownloaded}
    <div class="action-btn-group">
      <button
        class="btn-card-action hero"
        onclick={() => onDownloadAndInstall(updateId)}
        disabled={rebootRequired}
        title={rebootRequired
          ? "A system restart is required before updates can be installed."
          : "Download and immediately install this update"}
      >
        Download & Install
      </button>
      <button
        class="btn-card-action secondary"
        onclick={() => onDownload(updateId)}
        title="Download update package"
      >
        Download
      </button>
    </div>
  {:else if rebootRequired}
    <div class="action-btn-group">
      <button
        class="btn-card-action disabled-reboot"
        disabled
        title="A system restart is required before this update can be installed."
      >
        Restart Required
      </button>
    </div>
  {:else}
    <div class="action-btn-group">
      <button
        class="btn-card-action accent"
        onclick={() => onInstall(updateId)}
        title="Install this downloaded update"
      >
        Install Now
      </button>
    </div>
  {/if}
</div>

<style>
  .update-card-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .action-btn-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 140px;
  }

  .btn-card-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0.38rem 0.8rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .btn-card-action.hero {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-card-action.hero:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-card-action.secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-card-action.secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover);
  }

  .btn-card-action.accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-card-action.accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-card-action.cancel-queue {
    background-color: transparent;
    border: solid 1px var(--color-warning);
    color: var(--color-warning);
    font-size: 0.74rem;
  }

  .btn-card-action.cancel-queue:hover {
    background-color: var(--color-warning);
    color: var(--color-base);
  }

  .btn-card-action.disabled-reboot {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-warning);
    color: var(--color-warning);
    opacity: 0.8;
    cursor: not-allowed;
  }

  .btn-card-action:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .card-progress-indicator {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 155px;
    padding: 0.38rem 0.65rem;
    border-radius: 0.35rem;
    background-color: color-mix(in srgb, var(--color-primary) 12%, var(--color-surface));
    border: solid 1px color-mix(in srgb, var(--color-primary) 40%, transparent);
  }

  .card-progress-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .card-progress-label-group {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .mini-spinner {
    width: 10px;
    height: 10px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .card-progress-label {
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .card-progress-percent {
    font-size: 0.74rem;
    font-weight: 700;
    color: var(--color-primary);
    font-variant-numeric: tabular-nums;
  }

  .card-progress-track {
    width: 100%;
    height: 5px;
    background-color: color-mix(in srgb, var(--color-primary) 25%, transparent);
    border-radius: 3px;
    overflow: hidden;
  }

  .card-progress-fill {
    height: 100%;
    background-color: var(--color-primary);
    border-radius: 3px;
    transition: width 0.25s ease-out;
  }

  .card-progress-fill.indeterminate {
    background: linear-gradient(
      90deg,
      var(--color-primary) 0%,
      color-mix(in srgb, var(--color-primary) 40%, white) 50%,
      var(--color-primary) 100%
    );
    background-size: 200% 100%;
    animation: shimmer-progress 1.5s infinite linear;
  }

  @keyframes shimmer-progress {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .card-progress-bytes {
    font-size: 0.68rem;
    color: var(--color-muted);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
