<script lang="ts">
  import { onMount } from "svelte";
  import { formatBytes } from "../endpoint/metrics";
  import BreadcrumbsBar from "./components/breadcrumbs-bar.svelte";
  import DeleteConfirmModal from "./components/delete-confirm-modal.svelte";
  import FileTable from "./components/file-table.svelte";
  import FilterBuilderModal from "./components/filter-builder-modal.svelte";
  import ProgressBanner from "./components/progress-banner.svelte";
  import SavedFiltersDrawer from "./components/saved-filters-drawer.svelte";
  import VolumeOverview from "./components/volume-overview.svelte";
  import { filesystemStore } from "./filesystem.svelte";
  import type { FileEntry, FilterCriterion, SavedFilter } from "./types";

  const store = filesystemStore;

  let isFilterModalOpen = $state(false);
  let isSavedFiltersOpen = $state(false);
  let isDeleteConfirmOpen = $state(false);

  onMount(() => {
    store.init();
  });

  function handleOpenEntry(entry: FileEntry) {
    if (entry.is_dir) {
      store.navigateTo(entry.path);
    }
  }

  function handleApplyCustomFilter(criteria: FilterCriterion[], matchAll: boolean) {
    store.runSearch(criteria, matchAll);
  }

  function handleApplySavedFilter(filter: SavedFilter) {
    store.runSearch(filter.criteria, filter.match_all);
  }
</script>

<svelte:head>
  <title>Caffeine - File System</title>
</svelte:head>

<div class="filesystem-page">
  <div class="page-header">
    <h2 class="page-title">File System Analysis & Pruning</h2>
    {#if store.selectedVolume}
      <span class="active-volume-tag">
        Drive: {store.selectedVolume.mount_point}
      </span>
    {/if}
  </div>

  <VolumeOverview
    volumes={store.volumes}
    selectedVolume={store.selectedVolume}
    onSelectVolume={(vol) => store.selectVolume(vol)}
  />

  <BreadcrumbsBar
    currentPath={store.currentPath}
    isSearching={store.isSearching}
    searchResultsCount={store.searchResults ? store.searchResults.length : null}
    onNavigate={(path) => store.navigateTo(path)}
    onNavigateUp={() => store.navigateUp()}
    onRefresh={() => store.navigateTo(store.currentPath)}
    onOpenFilterModal={() => (isFilterModalOpen = true)}
    onOpenSavedFilters={() => (isSavedFiltersOpen = true)}
    onClearSearch={() => store.clearSearch()}
  />

  <ProgressBanner
    progress={store.progress}
    isSearching={store.isSearching}
    onCancel={() => store.cancelActiveTask()}
  />

  {#if store.error}
    <div class="error-banner" role="alert">
      <span>{store.error}</span>
      <button type="button" class="btn-dismiss" onclick={() => (store.error = null)}>
        Dismiss
      </button>
    </div>
  {/if}

  {#if store.deleteSummary}
    <div class="summary-banner" role="status">
      <span>
        Successfully deleted {store.deleteSummary.succeeded_paths.length} items
        (reclaimed {formatBytes(store.deleteSummary.total_bytes_freed)}).
      </span>
      {#if store.deleteSummary.failed_paths.length > 0}
        <span class="failed-note">
          {store.deleteSummary.failed_paths.length} items could not be removed.
        </span>
      {/if}
      <button
        type="button"
        class="btn-dismiss"
        onclick={() => (store.deleteSummary = null)}
      >
        &times;
      </button>
    </div>
  {/if}

  <FileTable
    entries={store.displayedEntries}
    selectedPaths={store.selectedPaths}
    isLoading={store.isLoading}
    onToggleSelect={(path) => store.toggleSelect(path)}
    onSelectAll={() => store.selectAll()}
    onDeselectAll={() => store.deselectAll()}
    onOpenEntry={handleOpenEntry}
  />

  {#if store.selectedPaths.size > 0}
    <div class="selection-floating-bar" role="toolbar" aria-label="Selection actions">
      <div class="selection-stats">
        <span class="selection-count">
          {store.selectedPaths.size} {store.selectedPaths.size === 1 ? 'item' : 'items'} selected
        </span>
        <span class="selection-bytes">
          ({formatBytes(store.totalSelectedBytes)})
        </span>
      </div>

      <div class="selection-actions">
        <button
          type="button"
          class="btn-secondary"
          onclick={() => store.deselectAll()}
        >
          Deselect All
        </button>
        <button
          type="button"
          class="btn-danger"
          onclick={() => (isDeleteConfirmOpen = true)}
        >
          Delete Selected
        </button>
      </div>
    </div>
  {/if}
</div>

<FilterBuilderModal
  bind:open={isFilterModalOpen}
  onApply={handleApplyCustomFilter}
  onSave={async (f) => {
    await store.saveFilter(f);
  }}
/>

<SavedFiltersDrawer
  bind:open={isSavedFiltersOpen}
  filters={store.savedFilters}
  onApply={handleApplySavedFilter}
  onDelete={(id) => store.deleteFilter(id)}
/>

<DeleteConfirmModal
  bind:open={isDeleteConfirmOpen}
  paths={Array.from(store.selectedPaths)}
  totalBytes={store.totalSelectedBytes}
  isDeleting={store.isDeleting}
  onConfirm={() => store.deleteSelected()}
/>

<style>
  .filesystem-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 1rem 1.25rem;
    box-sizing: border-box;
    overflow: hidden;
    position: relative;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .page-title {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0;
  }

  .active-volume-tag {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    background-color: var(--color-surface);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    border: solid var(--width-border) var(--color-border);
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.625rem 0.875rem;
    border-radius: 0.375rem;
    background-color: rgba(239, 68, 68, 0.1);
    border: solid var(--width-border) rgba(239, 68, 68, 0.3);
    color: var(--color-failure, #ef4444);
    font-size: 0.8125rem;
    margin-bottom: 0.75rem;
  }

  .summary-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    background-color: rgba(34, 197, 94, 0.1);
    border: solid var(--width-border) rgba(34, 197, 94, 0.3);
    color: var(--color-success, #22c55e);
    font-size: 0.8125rem;
    margin-bottom: 0.75rem;
  }

  .failed-note {
    color: var(--color-failure, #ef4444);
  }

  .btn-dismiss {
    margin-left: auto;
    border: none;
    background: transparent;
    color: inherit;
    font-size: 1rem;
    cursor: pointer;
  }

  .selection-floating-bar {
    position: absolute;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.625rem 1.25rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-primary);
    border-radius: 0.5rem;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.25);
    z-index: 10;
  }

  .selection-stats {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
  }

  .selection-count {
    font-weight: 600;
  }

  .selection-bytes {
    color: var(--color-text-secondary);
  }

  .selection-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn-secondary {
    padding: 0.3125rem 0.625rem;
    font-size: 0.75rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-danger {
    padding: 0.3125rem 0.625rem;
    font-size: 0.75rem;
    border: none;
    border-radius: 0.25rem;
    background-color: var(--color-failure, #ef4444);
    color: #ffffff;
    cursor: pointer;
    font-weight: 500;
  }
</style>
