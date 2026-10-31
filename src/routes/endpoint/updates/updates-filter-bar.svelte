<script lang="ts">
  import type { UpdateFilterCategory, UpdateSortOption } from "../types";

  interface Props {
    activeFilter: UpdateFilterCategory;
    onFilterChange: (cat: UpdateFilterCategory) => void;
    totalCount: number;
    pendingDownloadCount: number;
    readyToInstallCount: number;
    mandatoryCount: number;
    searchQuery: string;
    onSearchChange: (q: string) => void;
    sortOption: UpdateSortOption;
    onSortChange: (opt: UpdateSortOption) => void;
  }

  let {
    activeFilter,
    onFilterChange,
    totalCount,
    pendingDownloadCount,
    readyToInstallCount,
    mandatoryCount,
    searchQuery,
    onSearchChange,
    sortOption,
    onSortChange,
  }: Props = $props();
</script>

<div class="filter-controls-row">
  <div class="filter-tabs">
    <button
      class="filter-tab"
      class:active={activeFilter === "all"}
      onclick={() => onFilterChange("all")}
    >
      All ({totalCount})
    </button>
    <button
      class="filter-tab"
      class:active={activeFilter === "pending_download"}
      onclick={() => onFilterChange("pending_download")}
    >
      Pending ({pendingDownloadCount})
    </button>
    <button
      class="filter-tab"
      class:active={activeFilter === "ready_to_install"}
      onclick={() => onFilterChange("ready_to_install")}
    >
      Ready ({readyToInstallCount})
    </button>
    {#if mandatoryCount > 0}
      <button
        class="filter-tab"
        class:active={activeFilter === "mandatory"}
        onclick={() => onFilterChange("mandatory")}
      >
        Mandatory ({mandatoryCount})
      </button>
    {/if}
  </div>

  <div class="search-and-sort">
    <div class="search-input-wrap">
      <svg
        class="search-icon"
        xmlns="http://www.w3.org/2000/svg"
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search updates, KB, categories..."
        value={searchQuery}
        oninput={(e) => onSearchChange((e.target as HTMLInputElement).value)}
      />
      {#if searchQuery}
        <button class="btn-clear-search" onclick={() => onSearchChange("")}>&times;</button>
      {/if}
    </div>

    <div class="sort-wrap">
      <label for="sort-select" class="sort-label">Sort:</label>
      <select
        id="sort-select"
        class="sort-select"
        value={sortOption}
        onchange={(e) => onSortChange((e.target as HTMLSelectElement).value as UpdateSortOption)}
      >
        <option value="severity">Severity</option>
        <option value="size">Size</option>
        <option value="title">Title</option>
        <option value="kb">KB Number</option>
      </select>
    </div>
  </div>
</div>

<style>
  .filter-controls-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .filter-tabs {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .filter-tab {
    background: transparent;
    border: solid 1px transparent;
    padding: 0.25rem 0.55rem;
    border-radius: 0.25rem;
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .filter-tab:hover {
    color: var(--color-text);
    background-color: var(--color-base);
  }

  .filter-tab.active {
    color: var(--color-primary);
    background-color: color-mix(in srgb, var(--color-primary) 12%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-primary) 35%, transparent);
  }

  .search-and-sort {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  :global(.search-icon) {
    position: absolute;
    left: 0.5rem;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .search-input {
    padding: 0.28rem 1.6rem 0.28rem 1.7rem;
    font-size: 0.76rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    color: var(--color-text);
    width: 220px;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--color-primary);
  }

  .btn-clear-search {
    position: absolute;
    right: 0.4rem;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
    padding: 0;
  }

  .sort-wrap {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .sort-label {
    font-size: 0.74rem;
    color: var(--color-text-muted);
  }

  .sort-select {
    padding: 0.25rem 0.45rem;
    font-size: 0.74rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    color: var(--color-text);
    cursor: pointer;
    outline: none;
  }

  .sort-select:focus {
    border-color: var(--color-primary);
  }
</style>
