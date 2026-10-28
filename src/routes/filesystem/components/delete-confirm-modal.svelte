<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import { formatBytes } from "../../endpoint/metrics";

  interface Props {
    open: boolean;
    paths: string[];
    totalBytes: number;
    isDeleting: boolean;
    onConfirm: () => Promise<void>;
  }

  let {
    open = $bindable(false),
    paths,
    totalBytes,
    isDeleting,
    onConfirm,
  }: Props = $props();

  async function handleDelete() {
    await onConfirm();
    open = false;
  }
</script>

<Modal bind:open maxWidth="36rem" ariaLabel="Confirm Deletion">
  <div class="confirm-content">
    <div class="modal-header">
      <h3 class="modal-title">Delete Selected Items?</h3>
      <button
        type="button"
        class="btn-close"
        onclick={() => (open = false)}
        disabled={isDeleting}
        aria-label="Close"
      >
        &times;
      </button>
    </div>

    <div class="warning-banner">
      <strong>Warning:</strong> This operation will permanently remove the selected
      items from your file system.
    </div>

    <div class="summary-line">
      <span>Items to delete: <strong>{paths.length}</strong></span>
      <span>Space to be reclaimed: <strong>{formatBytes(totalBytes)}</strong></span>
    </div>

    <div class="paths-preview-list">
      {#each paths as p}
        <div class="path-item" title={p}>{p}</div>
      {/each}
    </div>

    <div class="modal-footer">
      <button
        type="button"
        class="btn-secondary"
        onclick={() => (open = false)}
        disabled={isDeleting}
      >
        Cancel
      </button>
      <button
        type="button"
        class="btn-danger"
        onclick={handleDelete}
        disabled={isDeleting}
      >
        {#if isDeleting}
          Deleting...
        {:else}
          Delete {paths.length} {paths.length === 1 ? 'Item' : 'Items'}
        {/if}
      </button>
    </div>
  </div>
</Modal>

<style>
  .confirm-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    color: var(--color-text);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-title {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .btn-close {
    border: none;
    background: transparent;
    font-size: 1.25rem;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .warning-banner {
    padding: 0.625rem 0.75rem;
    border-radius: 0.375rem;
    background-color: rgba(239, 68, 68, 0.1);
    border: solid var(--width-border) rgba(239, 68, 68, 0.3);
    color: var(--color-failure, #ef4444);
    font-size: 0.8125rem;
  }

  .summary-line {
    display: flex;
    justify-content: space-between;
    font-size: 0.8125rem;
  }

  .paths-preview-list {
    max-height: 180px;
    overflow-y: auto;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .path-item {
    font-size: 0.75rem;
    font-family: monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text-muted);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .btn-secondary {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-danger {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: none;
    border-radius: 0.25rem;
    background-color: var(--color-failure, #ef4444);
    color: #ffffff;
    cursor: pointer;
    font-weight: 500;
  }

  .btn-danger:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-danger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
