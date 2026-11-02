<script lang="ts">
  import { formatBytes } from "../metrics";

  interface Props {
    selectedCount: number;
    selectedBytes: number;
    selectedPendingCount: number;
    selectedReadyCount: number;
    isBusy: boolean;
    isRebootRequired: boolean;
    onBatchDownloadAndInstall: () => void;
    onBatchDownload: () => void;
    onBatchInstall: () => void;
    onClearSelection: () => void;
  }

  let {
    selectedCount,
    selectedBytes,
    selectedPendingCount,
    selectedReadyCount,
    isBusy,
    isRebootRequired,
    onBatchDownloadAndInstall,
    onBatchDownload,
    onBatchInstall,
    onClearSelection,
  }: Props = $props();
</script>

<div class="batch-selection-bar">
  <div class="batch-info">
    <span class="batch-count">
      <strong>{selectedCount}</strong> update(s) selected
    </span>
    {#if selectedBytes > 0}
      <span class="batch-size">({formatBytes(selectedBytes)})</span>
    {/if}
  </div>

  <div class="batch-actions">
    <button
      class="btn-batch-primary"
      onclick={onBatchDownloadAndInstall}
      disabled={isBusy || isRebootRequired}
      title={isRebootRequired
        ? "A system restart is required before installing updates."
        : "Download and install selected updates"}
    >
      Download & Install Selected ({selectedCount})
    </button>

    {#if selectedPendingCount > 0}
      <button
        class="btn-batch-secondary"
        onclick={onBatchDownload}
        disabled={isBusy}
        title="Download selected pending updates"
      >
        Download Selected ({selectedPendingCount})
      </button>
    {/if}

    {#if selectedReadyCount > 0}
      <button
        class="btn-batch-accent"
        onclick={onBatchInstall}
        disabled={isBusy || isRebootRequired}
        title={isRebootRequired
          ? "A system restart is required before installing updates."
          : "Install selected downloaded updates"}
      >
        Install Selected ({selectedReadyCount})
      </button>
    {/if}

    <button class="btn-batch-deselect" onclick={onClearSelection}>
      Deselect All
    </button>
  </div>
</div>

<style>
  .batch-selection-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-primary) 8%, var(--color-surface));
    border: solid var(--width-border) color-mix(in srgb, var(--color-primary) 30%, transparent);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
    gap: 0.75rem;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .batch-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--color-text);
  }

  .batch-count strong {
    color: var(--color-primary);
  }

  .batch-size {
    color: var(--color-text-muted);
  }

  .batch-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .btn-batch-primary,
  .btn-batch-secondary,
  .btn-batch-accent,
  .btn-batch-deselect {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.28rem 0.6rem;
    border-radius: 0.25rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-batch-primary {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-batch-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-batch-secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-batch-secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .btn-batch-accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-batch-accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-batch-deselect {
    background: transparent;
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
  }

  .btn-batch-deselect:hover {
    color: var(--color-text);
    border-color: var(--color-text);
  }

  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
