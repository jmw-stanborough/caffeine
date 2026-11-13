<script lang="ts">
  import { formatRelativeTime } from "$lib/format";
  import type { ScriptSummary } from "./types";

  interface Props {
    scripts: ScriptSummary[];
    activeId: string | null;
    loading: boolean;
    onSelect: (id: string) => void;
    onCreateNew: () => void;
  }

  let { scripts, activeId, loading, onSelect, onCreateNew }: Props = $props();

  let searchQuery = $state("");

  let filteredScripts = $derived.by(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return scripts;
    return scripts.filter(
      (script) =>
        script.name.toLowerCase().includes(query) ||
        script.filename.toLowerCase().includes(query) ||
        script.extension.toLowerCase().includes(query),
    );
  });
</script>

<aside class="scripts-sidebar">
  <div class="sidebar-top">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Scripts</h2>
      <button type="button" class="btn-new" onclick={onCreateNew}>
        + New Script
      </button>
    </div>

    <input
      type="search"
      class="search-input"
      placeholder="Search scripts..."
      aria-label="Search scripts"
      bind:value={searchQuery}
    />
  </div>

  <div class="scripts-list">
    {#if loading && scripts.length === 0}
      <p class="empty-hint">Loading scripts...</p>
    {:else if filteredScripts.length === 0}
      <p class="empty-hint">
        {searchQuery.trim() ? "No matching scripts." : "No scripts yet. Create one to run in terminal."}
      </p>
    {:else}
      {#each filteredScripts as script (script.id)}
        <button
          type="button"
          class="script-item"
          class:active={script.id === activeId}
          onclick={() => onSelect(script.id)}
        >
          <div class="script-item-header">
            <span class="script-item-name">{script.name}</span>
            <span class="ext-badge">.{script.extension}</span>
          </div>
          <div class="script-item-meta">
            <span class="filename-label">{script.filename}</span>
            <span>{formatRelativeTime(new Date(script.updated_at_ms).toISOString())}</span>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .scripts-sidebar {
    width: 17.5rem;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background-color: var(--color-surface);
    border-right: solid var(--width-border) var(--color-border);
    height: 100%;
    min-height: 0;
  }

  .sidebar-top {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 0.9rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .sidebar-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .btn-new {
    background-color: var(--color-primary);
    border: solid var(--width-border) var(--color-primary);
    color: var(--color-base);
    padding: 0.35rem 0.65rem;
    border-radius: 0.375rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-new:hover {
    filter: brightness(1.08);
  }

  .search-input {
    width: 100%;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.4rem 0.6rem;
    color: var(--color-text);
    font-size: 0.84rem;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--color-primary);
  }

  .scripts-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .empty-hint {
    margin: 1rem 0.5rem;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    text-align: center;
  }

  .script-item {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    text-align: left;
    background: transparent;
    border: solid var(--width-border) transparent;
    border-radius: 0.375rem;
    padding: 0.55rem 0.65rem;
    cursor: pointer;
    color: var(--color-text);
    transition: background-color 0.12s ease, border-color 0.12s ease;
  }

  .script-item:hover {
    background-color: var(--color-surface-hover);
  }

  .script-item.active {
    background-color: color-mix(in srgb, var(--color-primary) 16%, var(--color-base));
    border-color: var(--color-primary);
  }

  .script-item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .script-item-name {
    font-size: 0.88rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ext-badge {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--color-primary);
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.1rem 0.35rem;
    flex-shrink: 0;
  }

  .script-item-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .filename-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
