<script lang="ts">
  import { formatBytes } from "../../endpoint/metrics";
  import type { OperationProgress } from "../types";

  interface Props {
    progress: OperationProgress | null;
    isSearching: boolean;
    onCancel: () => void;
  }

  let { progress, isSearching, onCancel }: Props = $props();
</script>

{#if isSearching}
  <div class="progress-banner" role="status" aria-live="polite">
    <div class="progress-main-row">
      <div class="status-wrap">
        <span class="spinner"></span>
        <span class="status-text">
          Searching file system...
          {#if progress}
            ({progress.items_processed.toLocaleString()} items scanned
            {#if progress.bytes_processed > 0}
              &bull; {formatBytes(progress.bytes_processed)}
            {/if})
          {/if}
        </span>
      </div>

      <button type="button" class="btn-cancel" onclick={onCancel}>
        Cancel
      </button>
    </div>

    {#if progress?.current_path}
      <div class="path-preview" title={progress.current_path}>
        {progress.current_path}
      </div>
    {/if}

    <div class="progress-track" role="progressbar" aria-label="Search progress">
      <div class="progress-indeterminate"></div>
    </div>
  </div>
{/if}

<style>
  .progress-banner {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding: 0.625rem 0.875rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-primary);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
  }

  .progress-main-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .path-preview {
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .progress-track {
    width: 100%;
    height: 3px;
    background-color: var(--color-background);
    border-radius: 999px;
    overflow: hidden;
    position: relative;
  }

  .progress-indeterminate {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 30%;
    background-color: var(--color-primary);
    border-radius: 999px;
    animation: indeterminate 1.5s infinite ease-in-out;
  }

  @keyframes indeterminate {
    0% {
      left: -30%;
    }
    100% {
      left: 100%;
    }
  }

  .btn-cancel {
    padding: 0.1875rem 0.5rem;
    font-size: 0.75rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-background);
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .btn-cancel:hover {
    background-color: var(--color-surface-hover, rgba(255, 255, 255, 0.06));
  }
</style>
