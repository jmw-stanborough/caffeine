<script lang="ts">
  import { FileText, Folder } from "$lib/components/icons";
  import { formatBytes } from "../../endpoint/metrics";
  import type { FileEntry } from "../types";

  interface Props {
    entries: FileEntry[];
    selectedPaths: Set<string>;
    isLoading: boolean;
    onToggleSelect: (path: string) => void;
    onSelectAll: () => void;
    onDeselectAll: () => void;
    onOpenEntry: (entry: FileEntry) => void;
  }

  let {
    entries,
    selectedPaths,
    isLoading,
    onToggleSelect,
    onSelectAll,
    onDeselectAll,
    onOpenEntry,
  }: Props = $props();

  let allSelected = $derived(
    entries.length > 0 && entries.every((e) => selectedPaths.has(e.path)),
  );
  let someSelected = $derived(
    entries.some((e) => selectedPaths.has(e.path)) && !allSelected,
  );

  function getUsageBarColor(percent: number): string {
    if (percent < 5) return "var(--color-success, #22c55e)";
    if (percent < 20) return "var(--color-warning, #eab308)";
    return "var(--color-failure, #ef4444)";
  }

  function handleMasterCheckbox() {
    if (allSelected) {
      onDeselectAll();
    } else {
      onSelectAll();
    }
  }
</script>

<div class="table-container">
  <table class="file-table">
    <thead>
      <tr>
        <th class="th-checkbox">
          <input
            type="checkbox"
            checked={allSelected}
            indeterminate={someSelected}
            onchange={handleMasterCheckbox}
            aria-label="Select all items"
          />
        </th>
        <th class="th-name">Name</th>
        <th class="th-size">Size</th>
        <th class="th-percent">% of Drive</th>
      </tr>
    </thead>
    <tbody>
      {#if isLoading}
        <tr>
          <td colspan="4" class="cell-empty">
            <div class="loading-state">
              <span class="spinner"></span>
              <span>Scanning directory contents...</span>
            </div>
          </td>
        </tr>
      {:else if entries.length === 0}
        <tr>
          <td colspan="4" class="cell-empty">
            <span>No files or folders found</span>
          </td>
        </tr>
      {:else}
        {#each entries as entry (entry.path)}
          {@const isSelected = selectedPaths.has(entry.path)}
          {@const color = getUsageBarColor(entry.drive_usage_percent)}
          {@const percentDisplay = entry.drive_usage_percent < 0.01 && entry.size_bytes > 0
            ? "<0.01%"
            : `${entry.drive_usage_percent.toFixed(2)}%`}

          <tr
            class="file-row"
            class:selected={isSelected}
            ondblclick={() => entry.is_dir && onOpenEntry(entry)}
          >
            <td class="td-checkbox">
              <input
                type="checkbox"
                checked={isSelected}
                onchange={() => onToggleSelect(entry.path)}
                aria-label={`Select ${entry.name}`}
              />
            </td>
            <td class="td-name">
              <button
                type="button"
                class="entry-name-btn"
                onclick={() => entry.is_dir && onOpenEntry(entry)}
              >
                {#if entry.is_dir}
                  <Folder size={16} class="folder-icon" />
                {:else}
                  <FileText size={16} class="file-icon" />
                {/if}
                <span class="name-text" title={entry.path}>{entry.name}</span>
                {#if entry.is_dir && entry.child_count !== null && entry.child_count !== undefined}
                  <span class="child-count">({entry.child_count})</span>
                {/if}
              </button>
            </td>
            <td class="td-size">
              {#if entry.is_dir && entry.size_bytes === 0}
                <span class="muted-dash">&mdash;</span>
              {:else}
                {formatBytes(entry.size_bytes)}
              {/if}
            </td>
            <td class="td-percent">
              {#if entry.is_dir && entry.size_bytes === 0}
                <span class="muted-dash">&mdash;</span>
              {:else}
                <div class="percent-wrap">
                  <span class="percent-label" style:color>{percentDisplay}</span>
                  <div class="mini-bar-track">
                    <div
                      class="mini-bar-fill"
                      style:width={`${Math.min(100, Math.max(1, entry.drive_usage_percent))}%`}
                      style:background-color={color}
                    ></div>
                  </div>
                </div>
              {/if}
            </td>
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-container {
    width: 100%;
    flex: 1;
    overflow-y: auto;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    background-color: var(--color-surface);
  }

  .file-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  thead {
    position: sticky;
    top: 0;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    z-index: 1;
  }

  th {
    padding: 0.5rem 0.75rem;
    text-align: left;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .th-checkbox,
  .td-checkbox {
    width: 32px;
    padding-left: 0.75rem;
    padding-right: 0.25rem;
  }

  .th-size,
  .td-size {
    width: 120px;
    text-align: right;
  }

  .th-percent,
  .td-percent {
    width: 160px;
  }

  .file-row {
    border-bottom: solid var(--width-border) var(--color-border);
    transition: background-color 0.1s ease;
  }

  .file-row:hover {
    background-color: var(--color-surface-hover, rgba(255, 255, 255, 0.04));
  }

  .file-row.selected {
    background-color: var(--color-primary-transparent, rgba(14, 165, 233, 0.1));
  }

  td {
    padding: 0.4375rem 0.75rem;
  }

  .entry-name-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    text-align: left;
    padding: 0;
    max-width: 100%;
  }

  .name-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .child-count {
    color: var(--color-text-secondary);
    font-size: 0.75rem;
  }

  .percent-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .muted-dash {
    color: var(--color-text-muted, #7c6f64);
  }

  .percent-label {
    min-width: 48px;
    text-align: right;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .mini-bar-track {
    flex: 1;
    height: 5px;
    background-color: var(--color-base);
    border-radius: 999px;
    overflow: hidden;
  }

  .mini-bar-fill {
    height: 100%;
    border-radius: 999px;
  }

  .cell-empty {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--color-text-secondary);
  }

  .loading-state {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .spinner {
    width: 16px;
    height: 16px;
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
</style>
