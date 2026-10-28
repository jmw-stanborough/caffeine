<script lang="ts">
  import { Filter } from "$lib/components/icons";

  interface Props {
    currentPath: string;
    isSearching: boolean;
    searchResultsCount: number | null;
    onNavigate: (path: string) => void;
    onNavigateUp: () => void;
    onRefresh: () => void;
    onOpenFilterModal: () => void;
    onOpenSavedFilters: () => void;
    onClearSearch: () => void;
  }

  let {
    currentPath,
    isSearching,
    searchResultsCount,
    onNavigate,
    onNavigateUp,
    onRefresh,
    onOpenFilterModal,
    onOpenSavedFilters,
    onClearSearch,
  }: Props = $props();

  let breadcrumbs = $derived.by(() => {
    if (!currentPath) return [];
    const isWindows = currentPath.includes(":\\");
    const sep = isWindows ? "\\" : "/";
    const parts = currentPath.split(/[/\\]+/).filter(Boolean);

    const crumbs: { label: string; fullPath: string }[] = [];
    let accumulator = isWindows ? "" : "/";

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      if (i === 0 && isWindows) {
        accumulator = part + "\\";
      } else {
        accumulator = isWindows
          ? accumulator + (accumulator.endsWith("\\") ? "" : "\\") + part
          : accumulator + (accumulator.endsWith("/") ? "" : "/") + part;
      }
      crumbs.push({ label: part, fullPath: accumulator });
    }

    return crumbs;
  });
</script>

<div class="breadcrumbs-toolbar">
  <div class="nav-actions">
    <button
      type="button"
      class="btn-icon"
      onclick={onNavigateUp}
      title="Navigate up one level"
      aria-label="Up"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="m18 15-6-6-6 6" />
      </svg>
    </button>

    <button
      type="button"
      class="btn-icon"
      onclick={onRefresh}
      title="Refresh directory contents"
      aria-label="Refresh"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="15"
        height="15"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
        <path d="M16 21h5v-5" />
      </svg>
    </button>
  </div>

  <nav class="breadcrumbs-list" aria-label="Breadcrumb path">
    {#each breadcrumbs as crumb, idx}
      <button
        type="button"
        class="crumb-btn"
        class:is-last={idx === breadcrumbs.length - 1}
        onclick={() => onNavigate(crumb.fullPath)}
      >
        {crumb.label}
      </button>
      {#if idx < breadcrumbs.length - 1}
        <span class="crumb-separator" aria-hidden="true">/</span>
      {/if}
    {/each}
  </nav>

  <div class="right-actions">
    {#if searchResultsCount !== null}
      <div class="search-badge">
        <span>{searchResultsCount} search {searchResultsCount === 1 ? 'match' : 'matches'}</span>
        <button
          type="button"
          class="btn-clear-search"
          onclick={onClearSearch}
          title="Clear search results"
          aria-label="Clear search"
        >
          &times;
        </button>
      </div>
    {/if}

    <button
      type="button"
      class="btn-secondary"
      onclick={onOpenSavedFilters}
      title="View saved filters"
    >
      <span>Saved Filters</span>
    </button>

    <button
      type="button"
      class="btn-primary"
      onclick={onOpenFilterModal}
      disabled={isSearching}
    >
      <Filter size={14} />
      <span>Filter & Search</span>
    </button>
  </div>
</div>

<style>
  .breadcrumbs-toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 0.25rem;
    border: solid var(--width-border) var(--color-border);
    background-color: var(--color-background);
    color: var(--color-text-primary);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .btn-icon:hover {
    background-color: var(--color-surface-hover, rgba(255, 255, 255, 0.06));
  }

  .breadcrumbs-list {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    overflow-x: auto;
    white-space: nowrap;
    scrollbar-width: none;
  }

  .breadcrumbs-list::-webkit-scrollbar {
    display: none;
  }

  .crumb-btn {
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font: inherit;
  }

  .crumb-btn:hover {
    color: var(--color-text-primary);
    background-color: var(--color-background);
  }

  .crumb-btn.is-last {
    color: var(--color-text-primary);
    font-weight: 600;
  }

  .crumb-separator {
    color: var(--color-text-secondary);
    opacity: 0.5;
    font-size: 0.75rem;
  }

  .right-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .search-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-primary-transparent, rgba(14, 165, 233, 0.15));
    color: var(--color-primary);
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .btn-clear-search {
    border: none;
    background: transparent;
    color: inherit;
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.3125rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-background);
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .btn-secondary:hover {
    background-color: var(--color-surface-hover, rgba(255, 255, 255, 0.06));
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.3125rem 0.625rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: none;
    border-radius: 0.25rem;
    background-color: var(--color-primary);
    color: #ffffff;
    cursor: pointer;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
