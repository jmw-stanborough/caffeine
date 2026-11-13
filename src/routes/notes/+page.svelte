<script lang="ts">
  import { formatTimestamp } from "$lib/format";
  import { onMount } from "svelte";
  import MarkdownViewer from "./markdown-viewer.svelte";
  import NotesSidebar from "./notes-sidebar.svelte";
  import { notesStore } from "./notes.svelte";
  import type { NoteViewMode } from "./types";

  let draftSlug = $state<string | null>(null);
  let draftTitle = $state("");
  let draftContent = $state("");
  let viewMode = $state<NoteViewMode>("split");
  let confirmingDelete = $state(false);

  let isDirty = $derived.by(() => {
    if (!notesStore.activeNote) {
      return draftTitle.trim().length > 0 || draftContent.length > 0;
    }
    return (
      draftTitle !== notesStore.activeNote.title ||
      draftContent !== notesStore.activeNote.content
    );
  });

  function syncDraftFromActive() {
    const active = notesStore.activeNote;
    draftSlug = active?.slug ?? null;
    draftTitle = active?.title ?? "";
    draftContent = active?.content ?? "";
    confirmingDelete = false;
  }

  async function handleSelectNote(slug: string) {
    await notesStore.selectNote(slug);
    syncDraftFromActive();
  }

  function handleCreateNew() {
    notesStore.activeNote = null;
    draftSlug = null;
    draftTitle = "";
    draftContent = "";
    confirmingDelete = false;
    if (viewMode === "read") {
      viewMode = "split";
    }
  }

  async function handleSave() {
    const trimmedTitle = draftTitle.trim();
    if (!trimmedTitle || notesStore.saving) return;

    const saved = await notesStore.saveNote(draftSlug, trimmedTitle, draftContent);
    draftSlug = saved.slug;
    draftTitle = saved.title;
    draftContent = saved.content;
    confirmingDelete = false;
  }

  async function handleDelete() {
    if (!draftSlug || notesStore.saving) return;
    if (!confirmingDelete) {
      confirmingDelete = true;
      return;
    }

    await notesStore.deleteNote(draftSlug);
    if (notesStore.notes.length > 0) {
      await handleSelectNote(notesStore.notes[0].slug);
    } else {
      handleCreateNew();
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      void handleSave();
    }
  }

  onMount(async () => {
    await notesStore.init();
    if (notesStore.activeNote) {
      syncDraftFromActive();
    } else if (notesStore.notes.length > 0) {
      await handleSelectNote(notesStore.notes[0].slug);
    }
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
  <title>Caffeine - Notes</title>
</svelte:head>

<div class="notes-layout">
  <NotesSidebar
    notes={notesStore.notes}
    activeSlug={draftSlug}
    loading={notesStore.loading}
    onSelect={handleSelectNote}
    onCreateNew={handleCreateNew}
  />

  <section class="notes-workspace">
    {#if notesStore.errorMessage}
      <div class="error-banner" role="alert">
        <span>{notesStore.errorMessage}</span>
        <button
          type="button"
          class="btn-retry"
          onclick={() => (notesStore.errorMessage = null)}
        >
          Dismiss
        </button>
      </div>
    {/if}

    <header class="editor-toolbar">
      <input
        type="text"
        class="title-input"
        placeholder="Note title..."
        aria-label="Note title"
        bind:value={draftTitle}
        oninput={() => (confirmingDelete = false)}
      />

      <div class="toolbar-controls">
        <div class="mode-group" role="group" aria-label="View mode">
          <button
            type="button"
            class="btn-toolbar"
            class:active={viewMode === "edit"}
            onclick={() => (viewMode = "edit")}
          >
            Edit
          </button>
          <button
            type="button"
            class="btn-toolbar"
            class:active={viewMode === "split"}
            onclick={() => (viewMode = "split")}
          >
            Split
          </button>
          <button
            type="button"
            class="btn-toolbar"
            class:active={viewMode === "read"}
            onclick={() => (viewMode = "read")}
          >
            Read
          </button>
        </div>

        <button
          type="button"
          class="btn-save"
          disabled={notesStore.saving || !draftTitle.trim() || (!isDirty && draftSlug !== null)}
          onclick={handleSave}
          title="Save note (Ctrl+S)"
        >
          {notesStore.saving ? "Saving..." : "Save"}
        </button>

        {#if draftSlug}
          <button
            type="button"
            class="btn-delete"
            class:confirming={confirmingDelete}
            disabled={notesStore.saving}
            onclick={handleDelete}
          >
            {confirmingDelete ? "Confirm Delete" : "Delete"}
          </button>
        {/if}
      </div>
    </header>

    <div class="editor-panes" class:split={viewMode === "split"}>
      {#if viewMode === "edit" || viewMode === "split"}
        <textarea
          class="markdown-editor"
          placeholder="Write Markdown notes here..."
          aria-label="Markdown note content"
          bind:value={draftContent}
          oninput={() => (confirmingDelete = false)}
        ></textarea>
      {/if}

      {#if viewMode === "read" || viewMode === "split"}
        <div class="viewer-pane">
          <MarkdownViewer content={draftContent} />
        </div>
      {/if}
    </div>

    <footer class="editor-status">
      <span class="status-badge">
        {notesStore.activeNote?.filename ?? "new-note.md"}
      </span>
      {#if isDirty}
        <span class="dirty-indicator">Unsaved changes</span>
      {/if}
      {#if notesStore.activeNote}
        <span class="status-updated">
          Updated {formatTimestamp(new Date(notesStore.activeNote.updated_at_ms).toISOString())}
        </span>
      {/if}
    </footer>
  </section>
</div>

<style>
  .notes-layout {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 0;
    background-color: var(--color-base);
  }

  .notes-workspace {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding: var(--page-padding);
    gap: 0.75rem;
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .title-input {
    flex: 1;
    min-width: 14rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.45rem 0.75rem;
    color: var(--color-text);
    font-size: 1rem;
    font-weight: 600;
    outline: none;
  }

  .title-input:focus {
    border-color: var(--color-primary);
  }

  .toolbar-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .mode-group {
    display: inline-flex;
    gap: 0.25rem;
  }

  .btn-save:disabled,
  .btn-delete:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .editor-panes {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.75rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
    background-color: var(--color-surface);
  }

  .editor-panes.split {
    grid-template-columns: 1fr 1fr;
  }

  .markdown-editor {
    width: 100%;
    height: 100%;
    min-height: 0;
    resize: none;
    border: none;
    outline: none;
    padding: 1rem 1.1rem;
    background-color: var(--color-base);
    color: var(--color-text);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.9rem;
    line-height: 1.6;
    tab-size: 2;
  }

  .viewer-pane {
    min-width: 0;
    min-height: 0;
    height: 100%;
    border-left: solid var(--width-border) var(--color-border);
  }

  .editor-panes:not(.split) .viewer-pane {
    border-left: none;
  }

  .editor-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.78rem;
    color: var(--color-text-muted);
  }

  .status-badge {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.12rem 0.45rem;
  }

  .dirty-indicator {
    color: var(--color-warning);
    font-weight: 600;
  }

  .status-updated {
    margin-left: auto;
  }
</style>
