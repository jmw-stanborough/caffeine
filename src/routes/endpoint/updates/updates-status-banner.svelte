<script lang="ts">
  import { formatBytes } from "../metrics";
  import type { UpdateProgressEvent } from "../types";

  interface Props {
    operationStatus: string;
    updateProgress?: UpdateProgressEvent | null;
    queueLength: number;
    onClearQueue: () => void;
  }

  let { operationStatus, updateProgress, queueLength, onClearQueue }: Props = $props();
</script>

<div class="status-banner">
  <div class="status-content">
    <span class="mini-spinner"></span>
    <div class="status-text-block">
      <div class="status-title-row">
        <span>{operationStatus}</span>
        <span class="progress-percent-label">
          {updateProgress ? `${updateProgress.percent_complete}%` : ""}
        </span>
      </div>
      <div
        class="status-progress-track"
        role="progressbar"
        aria-valuenow={updateProgress?.percent_complete ?? 0}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          class="status-progress-fill"
          class:indeterminate={!updateProgress || updateProgress.percent_complete === 0}
          style:width="{updateProgress ? Math.max(updateProgress.percent_complete, 4) : 100}%"
        ></div>
      </div>
      <div class="status-progress-meta">
        {#if updateProgress && updateProgress.total_count > 1}
          <span>Processing update {updateProgress.current_index + 1} of {updateProgress.total_count}</span>
        {:else if updateProgress}
          <span>{updateProgress.phase === 'download' ? 'Downloading package...' : 'Installing package...'}</span>
        {:else}
          <span>Processing...</span>
        {/if}
        {#if updateProgress && updateProgress.phase === 'download' && updateProgress.bytes_downloaded != null && updateProgress.total_bytes}
          <span>{formatBytes(updateProgress.bytes_downloaded)} / {formatBytes(updateProgress.total_bytes)}</span>
        {/if}
      </div>
    </div>
  </div>
  {#if queueLength > 0}
    <div class="queue-info">
      <span class="queue-pill">{queueLength} queued</span>
      <button class="btn-clear-queue" onclick={onClearQueue}>
        Clear Queue
      </button>
    </div>
  {/if}
</div>

<style>
  .status-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.65rem 1rem;
    background-color: color-mix(in srgb, var(--color-primary) 12%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-primary) 35%, transparent);
    gap: 1rem;
    flex-wrap: wrap;
  }

  .status-content {
    display: flex;
    align-items: flex-start;
    gap: 0.65rem;
    flex: 1;
    min-width: 250px;
  }

  .status-text-block {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    flex: 1;
  }

  .status-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .progress-percent-label {
    font-size: 0.78rem;
    font-weight: 700;
    color: var(--color-primary);
    font-variant-numeric: tabular-nums;
  }

  .status-progress-track {
    width: 100%;
    height: 6px;
    background-color: color-mix(in srgb, var(--color-primary) 20%, transparent);
    border-radius: 3px;
    overflow: hidden;
  }

  .status-progress-fill {
    height: 100%;
    background-color: var(--color-primary);
    border-radius: 3px;
    transition: width 0.25s ease-out;
  }

  .status-progress-fill.indeterminate {
    background: linear-gradient(
      90deg,
      var(--color-primary) 0%,
      color-mix(in srgb, var(--color-primary) 35%, white) 50%,
      var(--color-primary) 100%
    );
    background-size: 200% 100%;
    animation: shimmer-progress 1.5s infinite linear;
  }

  @keyframes shimmer-progress {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .status-progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  .mini-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--color-primary);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
    margin-top: 0.15rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .queue-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .queue-pill {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px color-mix(in srgb, var(--color-warning) 40%, transparent);
  }

  .btn-clear-queue {
    background: transparent;
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
    font-size: 0.72rem;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .btn-clear-queue:hover {
    border-color: var(--color-primary);
    color: var(--color-text);
  }
</style>
