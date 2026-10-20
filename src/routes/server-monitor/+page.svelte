<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import ServerCard from "./server-card.svelte";
  import ServerModal from "./server-modal.svelte";
  import { ServerStore } from "./servers.svelte";
  import type { Server } from "./types";

  const appWindow = getCurrentWindow();
  const store = new ServerStore();

  let isModalOpen = $state(false);
  let editingServer = $state<Server | undefined>(undefined);
  let isNocMode = $state(false);
  let cardScale = $state(1);

  function adjustScale(delta: number) {
    cardScale = Math.min(1.6, Math.max(0.7, +(cardScale + delta).toFixed(2)));
  }

  async function setNocMode(active: boolean) {
    isNocMode = active;
    try {
      await appWindow.setFullscreen(active);
    } catch (err) {
      console.error("Failed to set fullscreen:", err);
    }
  }

  function openEditModal(server: Server) {
    if (isNocMode) return;
    editingServer = server;
    isModalOpen = true;
  }

  onMount(() => {
    store.init();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isNocMode) setNocMode(false);
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      store.destroy();
      window.removeEventListener("keydown", handleKeyDown);
      appWindow.setFullscreen(false).catch(() => {});
    };
  });
</script>

<div
  class="page-container"
  class:noc-layout={isNocMode}
  style:--card-min-width="{18 * cardScale}rem"
  style:--noc-min-width="{16 * cardScale}rem"
  style:--card-min-height="{8.5 * cardScale}rem"
  style:--card-font-scale={cardScale}
>
  <header class="page-header">
    <div class="header-titles">
      <h1 class="page-title">{isNocMode ? "NOC Monitor" : "Servers"}</h1>
      <span class="server-count">{store.items.length} configured</span>
    </div>
    <div class="header-actions">
      <div class="zoom-controls">
        <button
          type="button"
          class="btn-zoom"
          onclick={() => adjustScale(-0.1)}
          aria-label="Decrease card size"
        >
          −
        </button>
        <button
          type="button"
          class="btn-zoom"
          onclick={() => adjustScale(0.1)}
          aria-label="Increase card size"
        >
          +
        </button>
      </div>

      <button
        type="button"
        class="btn-noc"
        onclick={() => setNocMode(!isNocMode)}
      >
        {isNocMode ? "Exit NOC (Esc)" : "NOC Mode"}
      </button>
    </div>
  </header>

  {#if store.errorMessage}
    <div class="error-banner">
      <span>{store.errorMessage}</span>
      <button type="button" class="btn-retry" onclick={store.fetchServers}>Retry</button>
    </div>
  {/if}

  {#if store.loading}
    <p class="status-message">Reading configuration...</p>
  {:else}
    <section
      class="server-grid"
      class:noc-grid={isNocMode}
      use:dndzone={{
        items: store.items,
        flipDurationMs: 200,
        dragDisabled: isNocMode,
        dropTargetStyle: {}
      }}
      onconsider={(e) => (store.items = e.detail.items)}
      onfinalize={(e) => store.reorderServers(e.detail.items)}
    >
      {#each store.items as server (server.id)}
        <div animate:flip={{ duration: 200 }} class="drag-wrapper">
          <ServerCard
            {server}
            nocMode={isNocMode}
            status={store.statusMap[server.host]?.status ?? "checking"}
            latencyMs={store.statusMap[server.host]?.latencyMs}
            lastEvent={store.statusMap[server.host]?.lastEvent}
            lastEventAt={store.statusMap[server.host]?.lastEventAt}
            now={store.now}
            onclick={() => openEditModal(server)}
          />
        </div>
      {/each}

      {#if !isNocMode}
        <div class="add-card-wrapper" aria-disabled="true">
          <button
            class="add-server-card"
            type="button"
            onclick={() => { editingServer = undefined; isModalOpen = true; }}
            aria-label="Add new server"
          >
            <span class="plus-icon" aria-hidden="true">+</span>
            <span class="add-title">Add server</span>
          </button>
        </div>
      {/if}
    </section>
  {/if}
</div>

<ServerModal
  bind:open={isModalOpen}
  server={editingServer}
  onsave={(saved, id) => store.saveServer(saved, id)}
  ondelete={(id) => store.deleteServer(id)}
/>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.5rem;
  }

  .page-container.noc-layout {
    position: fixed;
    inset: 0;
    box-sizing: border-box;
    background-color: var(--color-base);
    padding: 2rem 2.5rem;
    overflow-y: auto;
    z-index: 50;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-titles {
    display: flex;
    align-items: baseline;
    gap: 1rem;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .page-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .server-count {
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .zoom-controls {
    display: flex;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    overflow: hidden;
  }

  .btn-zoom {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    padding: 0.35rem 0.65rem;
    font-size: 0.95rem;
    font-weight: 700;
    line-height: 1;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .btn-zoom:first-child {
    border-right: solid var(--width-border) var(--color-border);
  }

  .btn-zoom:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .btn-noc {
    background-color: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.35rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-noc:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--card-min-width, 18rem), 1fr));
    gap: 1rem;
    width: 100%;
    outline: none;
  }

  .server-grid.noc-grid {
    grid-template-columns: repeat(auto-fill, minmax(var(--noc-min-width, 16rem), 1fr));
    gap: 1.25rem;
  }

  .drag-wrapper,
  .add-card-wrapper {
    height: 100%;
    min-height: var(--card-min-height, 8.5rem);
    outline: none;
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-error) 15%, transparent);
    color: var(--color-error);
    border: solid var(--width-border) var(--color-error);
  }

  .btn-retry {
    background: transparent;
    border: solid var(--width-border) var(--color-error);
    color: var(--color-error);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .add-server-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    min-height: var(--card-min-height, 8.5rem);
    height: 100%;
    width: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 1rem;
    background-color: transparent;
    border: 2px dashed var(--color-border);
    border-radius: 0.5rem;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
  }

  .add-server-card:hover {
    background-color: color-mix(in srgb, var(--color-surface) 30%, transparent);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .add-server-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .plus-icon {
    font-size: calc(2rem * var(--card-font-scale, 1));
    font-weight: 300;
    line-height: 1;
  }

  .add-title {
    font-size: calc(1rem * var(--card-font-scale, 1));
    font-weight: 600;
  }

  .status-message {
    color: var(--color-text-muted);
    font-size: 0.95rem;
  }
</style>
