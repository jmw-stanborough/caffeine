<script lang="ts">
  import { formatRelativeTime } from "$lib/format";
  import type { NoteSummary } from "./types";

  interface Props {
    notes: NoteSummary[];
    activeSlug: string | null;
    loading: boolean;
    onSelect: (slug: string) => void;
    onCreateNew: () => void;
  }

  let { notes, activeSlug, loading, onSelect, onCreateNew }: Props = $props();

  let searchQuery = $state("");

  let filteredNotes = $derived.by(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return notes;
    return notes.filter(
      (note) =>
        note.title.toLowerCase().includes(query) ||
        note.filename.toLowerCase().includes(query),
    );
  });
</script>

<aside class="notes-sidebar">
  <div class="sidebar-top">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Notes</h2>
      <button type="button" class="btn-new" onclick={onCreateNew}>
        + New Note
      </button>
    </div>

    <input
      type="search"
      class="search-input"
      placeholder="Search notes..."
      aria-label="Search notes"
      bind:value={searchQuery}
    />
  </div>

  <div class="notes-list">
    {#if loading && notes.length === 0}
      <p class="empty-hint">Loading notes...</p>
    {:else if filteredNotes.length === 0}
      <p class="empty-hint">
        {searchQuery.trim() ? "No matching notes." : "No notes yet. Create one to get started."}
      </p>
    {:else}
      {#each filteredNotes as note (note.slug)}
        <button
          type="button"
          class="note-item"
          class:active={note.slug === activeSlug}
          onclick={() => onSelect(note.slug)}
        >
          <div class="note-item-title">{note.title}</div>
          <div class="note-item-meta">
            <span class="filename-badge">{note.filename}</span>
            <span class="updated-time">
              {formatRelativeTime(new Date(note.updated_at_ms).toISOString())}
            </span>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .notes-sidebar {
    width: 17rem;
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

  .notes-list {
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

  .note-item {
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

  .note-item:hover {
    background-color: var(--color-surface-hover);
  }

  .note-item.active {
    background-color: color-mix(in srgb, var(--color-primary) 16%, var(--color-base));
    border-color: var(--color-primary);
  }

  .note-item-title {
    font-size: 0.88rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-item-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .filename-badge {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.1rem 0.35rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 9.5rem;
  }

  .updated-time {
    flex-shrink: 0;
  }
</style>
