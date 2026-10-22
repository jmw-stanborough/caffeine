<script lang="ts">
  import DashboardHeader from "$lib/components/dashboard-header.svelte";
  import {
    DEFAULT_CARD_SCALE,
    DND_FLIP_DURATION_MS,
    SERVER_CARD_MIN_HEIGHT_REM,
    SERVER_CARD_MIN_WIDTH_REM,
    SERVER_NOC_MIN_WIDTH_REM,
  } from "$lib/constants";
  import { clampCardScale } from "$lib/format";
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
  let cardScale = $state(DEFAULT_CARD_SCALE);

  function adjustScale(delta: number) {
    cardScale = clampCardScale(cardScale, delta);
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
  style:--card-min-width="{SERVER_CARD_MIN_WIDTH_REM * cardScale}rem"
  style:--noc-min-width="{SERVER_NOC_MIN_WIDTH_REM * cardScale}rem"
  style:--card-min-height="{SERVER_CARD_MIN_HEIGHT_REM * cardScale}rem"
  style:--card-font-scale={cardScale}
>
  <DashboardHeader
    title={isNocMode ? "NOC Monitor" : "Servers"}
    countText="{store.items.length} configured"
    {isNocMode}
    errorMessage={store.errorMessage}
    onAdjustScale={adjustScale}
    onToggleNoc={() => setNocMode(!isNocMode)}
    onRetry={store.fetchServers}
  />

  {#if store.loading}
    <p class="status-message">Reading configuration...</p>
  {:else}
    <section
      class="server-grid"
      class:noc-grid={isNocMode}
      use:dndzone={{
        items: store.items,
        flipDurationMs: DND_FLIP_DURATION_MS,
        dragDisabled: isNocMode,
        dropTargetStyle: {},
      }}
      onconsider={(e) => (store.items = e.detail.items)}
      onfinalize={(e) => store.reorderServers(e.detail.items)}
    >
      {#each store.items as server (server.id)}
        {@const hostState = store.statusMap[server.id]}
        <div animate:flip={{ duration: DND_FLIP_DURATION_MS }} class="drag-wrapper">
          <ServerCard
            {server}
            nocMode={isNocMode}
            status={hostState?.status ?? "checking"}
            latencyMs={hostState?.latencyMs}
            lastEvent={hostState?.lastEvent}
            lastEventAt={hostState?.lastEventAt}
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
            onclick={() => {
              editingServer = undefined;
              isModalOpen = true;
            }}
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
  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--card-min-width, 18rem)), 1fr));
    gap: 1rem;
    width: 100%;
    outline: none;
  }

  .server-grid.noc-grid {
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--noc-min-width, 16rem)), 1fr));
    gap: 1.25rem;
  }

  .drag-wrapper,
  .add-card-wrapper {
    height: 100%;
    min-height: var(--card-min-height, 8.5rem);
    outline: none;
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
</style>
