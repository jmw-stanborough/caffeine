<script lang="ts">
  import { goto } from "$app/navigation";
  import { formatTimestamp } from "$lib/format";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import ScriptEditor from "./script-editor.svelte";
  import ScriptsSidebar from "./scripts-sidebar.svelte";
  import { scriptsStore } from "./scripts.svelte";

  type EndpointWithTerminalQueue = typeof endpointStore & {
    queueTerminalCommand?: (cmd: string) => void;
  };

  let draftId = $state<string | null>(null);
  let draftName = $state("");
  let draftLanguageId = $state("");
  let draftContent = $state("");
  let formatting = $state(false);
  let running = $state(false);
  let confirmingDelete = $state(false);

  let defaultLanguageId = $derived(scriptsStore.languages[0]?.id ?? "powershell");

  let isDirty = $derived.by(() => {
    if (!scriptsStore.activeScript) {
      return draftName.trim().length > 0 || draftContent.length > 0;
    }
    return (
      draftName !== scriptsStore.activeScript.name ||
      draftLanguageId !== scriptsStore.activeScript.language_id ||
      draftContent !== scriptsStore.activeScript.content
    );
  });

  function syncDraftFromActive() {
    const active = scriptsStore.activeScript;
    draftId = active?.id ?? null;
    draftName = active?.name ?? "";
    draftLanguageId = active?.language_id ?? defaultLanguageId;
    draftContent = active?.content ?? "";
    confirmingDelete = false;
  }

  async function handleSelectScript(id: string) {
    await scriptsStore.selectScript(id);
    syncDraftFromActive();
  }

  function handleCreateNew() {
    scriptsStore.activeScript = null;
    draftId = null;
    draftName = "";
    draftLanguageId = defaultLanguageId;
    draftContent = "";
    confirmingDelete = false;
  }

  async function handleSave() {
    const trimmedName = draftName.trim();
    if (!trimmedName || scriptsStore.saving) return null;

    const languageId = draftLanguageId || defaultLanguageId;
    const saved = await scriptsStore.saveScript(draftId, trimmedName, languageId, draftContent);
    syncDraftFromActive();
    return saved;
  }

  async function handleFormat() {
    if (formatting || !draftContent) return;
    formatting = true;
    try {
      const languageId = draftLanguageId || defaultLanguageId;
      draftContent = await scriptsStore.formatContent(languageId, draftContent);
      confirmingDelete = false;
    } finally {
      formatting = false;
    }
  }

  async function handleDelete() {
    if (!draftId || scriptsStore.saving) return;
    if (!confirmingDelete) {
      confirmingDelete = true;
      return;
    }

    await scriptsStore.deleteScript(draftId);
    if (scriptsStore.scripts.length > 0) {
      await handleSelectScript(scriptsStore.scripts[0].id);
    } else {
      handleCreateNew();
    }
  }

  async function handleRunInTerminal() {
    if (running || scriptsStore.saving || !draftName.trim()) return;
    running = true;
    try {
      let targetId = draftId;
      if (!targetId || isDirty) {
        const saved = await handleSave();
        if (!saved) return;
        targetId = saved.id;
      }

      const command = await scriptsStore.buildRunCommand(targetId);
      const endpoint = endpointStore as EndpointWithTerminalQueue;
      if (typeof endpoint.queueTerminalCommand === "function") {
        endpoint.queueTerminalCommand(command);
        await goto("/terminal");
      } else {
        await goto("/terminal");
        await invoke("endpoint_write_terminal", { data: command });
      }
    } finally {
      running = false;
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      void handleSave();
    }
  }

  onMount(async () => {
    await scriptsStore.init();
    if (!draftLanguageId) {
      draftLanguageId = defaultLanguageId;
    }
    if (scriptsStore.activeScript) {
      syncDraftFromActive();
    } else if (scriptsStore.scripts.length > 0) {
      await handleSelectScript(scriptsStore.scripts[0].id);
    }
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
  <title>Caffeine - Scripts</title>
</svelte:head>

<div class="scripts-layout">
  <ScriptsSidebar
    scripts={scriptsStore.scripts}
    activeId={draftId}
    loading={scriptsStore.loading}
    onSelect={handleSelectScript}
    onCreateNew={handleCreateNew}
  />

  <section class="scripts-workspace">
    {#if scriptsStore.errorMessage}
      <div class="error-banner" role="alert">
        <span>{scriptsStore.errorMessage}</span>
        <button
          type="button"
          class="btn-retry"
          onclick={() => (scriptsStore.errorMessage = null)}
        >
          Dismiss
        </button>
      </div>
    {/if}

    <header class="editor-toolbar">
      <input
        type="text"
        class="name-input"
        placeholder="Script name..."
        aria-label="Script name"
        bind:value={draftName}
        oninput={() => (confirmingDelete = false)}
      />

      <select
        class="language-select"
        aria-label="Script language"
        bind:value={draftLanguageId}
        onchange={() => (confirmingDelete = false)}
      >
        {#if scriptsStore.languages.length === 0}
          <option value="powershell">PowerShell (.ps1)</option>
        {:else}
          {#each scriptsStore.languages as lang (lang.id)}
            <option value={lang.id}>{lang.label}</option>
          {/each}
        {/if}
      </select>

      <div class="toolbar-actions">
        <button
          type="button"
          class="btn-toolbar"
          disabled={formatting || !draftContent}
          onclick={handleFormat}
        >
          {formatting ? "Formatting..." : "Format"}
        </button>

        <button
          type="button"
          class="btn-save"
          disabled={scriptsStore.saving || !draftName.trim() || (!isDirty && draftId !== null)}
          onclick={handleSave}
          title="Save script (Ctrl+S)"
        >
          {scriptsStore.saving ? "Saving..." : "Save"}
        </button>

        {#if draftId}
          <button
            type="button"
            class="btn-delete"
            class:confirming={confirmingDelete}
            disabled={scriptsStore.saving}
            onclick={handleDelete}
          >
            {confirmingDelete ? "Confirm Delete" : "Delete"}
          </button>
        {/if}

        <button
          type="button"
          class="btn-run"
          disabled={running || scriptsStore.saving || !draftName.trim()}
          onclick={handleRunInTerminal}
        >
          {running ? "Launching..." : "Run in Terminal"}
        </button>
      </div>
    </header>

    <ScriptEditor
      bind:value={draftContent}
      languageId={draftLanguageId || defaultLanguageId}
      onchange={() => (confirmingDelete = false)}
    />

    <footer class="editor-status">
      <span class="status-badge">
        {scriptsStore.activeScript?.filename ?? "unsaved-script"}
      </span>
      {#if scriptsStore.activeScript?.path}
        <span class="path-label selectable" title={scriptsStore.activeScript.path}>
          {scriptsStore.activeScript.path}
        </span>
      {/if}
      {#if isDirty}
        <span class="dirty-indicator">Unsaved changes</span>
      {/if}
      {#if scriptsStore.activeScript}
        <span class="status-updated">
          Updated {formatTimestamp(new Date(scriptsStore.activeScript.updated_at_ms).toISOString())}
        </span>
      {/if}
    </footer>
  </section>
</div>

<style>
  .scripts-layout {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 0;
    background-color: var(--color-base);
  }

  .scripts-workspace {
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
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .name-input {
    flex: 1;
    min-width: 12rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.45rem 0.75rem;
    color: var(--color-text);
    font-size: 0.96rem;
    font-weight: 600;
    outline: none;
  }

  .name-input:focus,
  .language-select:focus {
    border-color: var(--color-primary);
  }

  .language-select {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.45rem 0.65rem;
    color: var(--color-text);
    font-size: 0.85rem;
    outline: none;
    cursor: pointer;
  }

  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .btn-run {
    background-color: var(--color-secondary);
    border: solid var(--width-border) var(--color-secondary);
    color: var(--color-base);
    padding: 0.45rem 0.95rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-run:hover:not(:disabled) {
    filter: brightness(1.08);
  }

  .btn-toolbar:disabled,
  .btn-save:disabled,
  .btn-delete:disabled,
  .btn-run:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .editor-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.78rem;
    color: var(--color-text-muted);
    min-width: 0;
  }

  .status-badge {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.12rem 0.45rem;
    flex-shrink: 0;
  }

  .path-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-indicator {
    color: var(--color-warning);
    font-weight: 600;
    flex-shrink: 0;
  }

  .status-updated {
    margin-left: auto;
    flex-shrink: 0;
  }
</style>
