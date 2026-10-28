<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import type { SavedFilter } from "../types";

  interface Props {
    open: boolean;
    filters: SavedFilter[];
    onApply: (filter: SavedFilter) => void;
    onDelete: (id: string) => Promise<void>;
  }

  let { open = $bindable(false), filters, onApply, onDelete }: Props = $props();

  function formatRuleSummary(filter: SavedFilter): string {
    const parts = filter.criteria.map((c) => {
      const prop = c.property === "extension" ? "ext" : c.property;
      return `${prop} ${c.condition} "${c.value}"`;
    });
    return parts.join(filter.match_all ? " AND " : " OR ");
  }
</script>

<Modal bind:open maxWidth="36rem" ariaLabel="Saved Roaming Filters">
  <div class="saved-modal-content">
    <div class="modal-header">
      <h3 class="modal-title">Saved Roaming Filters</h3>
      <button
        type="button"
        class="btn-close"
        onclick={() => (open = false)}
        aria-label="Close"
      >
        &times;
      </button>
    </div>

    <div class="filter-list">
      {#if filters.length === 0}
        <div class="empty-state">No saved filters found.</div>
      {:else}
        {#each filters as filter (filter.id)}
          <div class="filter-card">
            <div class="filter-info">
              <div class="title-row">
                <span class="filter-name">{filter.name}</span>
                {#if filter.is_preset}
                  <span class="preset-tag">Preset</span>
                {/if}
              </div>
              <span class="filter-summary">{formatRuleSummary(filter)}</span>
            </div>

            <div class="actions-row">
              {#if !filter.is_preset}
                <button
                  type="button"
                  class="btn-delete"
                  onclick={() => onDelete(filter.id)}
                  title="Delete custom filter"
                  aria-label={`Delete ${filter.name}`}
                >
                  Delete
                </button>
              {/if}
              <button
                type="button"
                class="btn-apply"
                onclick={() => {
                  onApply(filter);
                  open = false;
                }}
              >
                Apply
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="modal-footer">
      <button type="button" class="btn-secondary" onclick={() => (open = false)}>
        Close
      </button>
    </div>
  </div>
</Modal>

<style>
  .saved-modal-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    color: var(--color-text-primary);
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
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .filter-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 350px;
    overflow-y: auto;
  }

  .filter-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.625rem 0.75rem;
    background-color: var(--color-background);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
  }

  .filter-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .filter-name {
    font-size: 0.875rem;
    font-weight: 600;
  }

  .preset-tag {
    font-size: 0.625rem;
    padding: 0.0625rem 0.375rem;
    background-color: rgba(14, 165, 233, 0.15);
    color: var(--color-primary);
    border-radius: 0.25rem;
    text-transform: uppercase;
  }

  .filter-summary {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn-apply {
    padding: 0.25rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: none;
    border-radius: 0.25rem;
    background-color: var(--color-primary);
    color: #ffffff;
    cursor: pointer;
  }

  .btn-delete {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: transparent;
    color: var(--color-failure, #ef4444);
    cursor: pointer;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding-top: 0.5rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .btn-secondary {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-background);
    color: var(--color-text-primary);
    cursor: pointer;
  }
</style>
