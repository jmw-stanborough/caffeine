<script lang="ts">
  import DashboardHeader from "$lib/components/dashboard-header.svelte";
  import SortModal from "$lib/components/sort-modal.svelte";
  import {
    DND_FLIP_DURATION_MS,
    SERVER_CARD_MIN_HEIGHT_REM,
    SERVER_CARD_MIN_WIDTH_REM,
    SERVER_NOC_MIN_WIDTH_REM,
  } from "$lib/constants";
  import { Circle, Sort } from "$lib/components/icons";
  import { createNocController } from "$lib/noc.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import { loadSavedSortRules, saveSortRules, sortEntities } from "$lib/sort/sort";
  import type { SortRule } from "$lib/sort/types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import ServerCard from "./server-card.svelte";
  import ServerModal from "./server-modal.svelte";
  import ServerTerminalDrawer from "./server-terminal-drawer.svelte";
  import { serverStore } from "./servers.svelte";
  import { SERVER_SORT_FIELDS, SERVER_SORT_STORAGE_KEY } from "./sort";
  import type { Server } from "./types";

  const store = serverStore;
  const noc = createNocController(() => {
    isModalOpen = false;
    isSortModalOpen = false;
    terminalServer = null;
  });

  let isModalOpen = $state(false);
  let isSortModalOpen = $state(false);
  let editingServer = $state<Server | undefined>(undefined);
  let terminalServer = $state<Server | null>(null);
  let sortRules = $state<SortRule[]>(loadSavedSortRules(SERVER_SORT_STORAGE_KEY));

  $effect(() => {
    saveSortRules(SERVER_SORT_STORAGE_KEY, sortRules);
  });

  let isSorted = $derived(sortRules.length > 0);
  let displayServers = $derived(
    isSorted
      ? sortEntities(
          store.items,
          sortRules,
          SERVER_SORT_FIELDS,
          store.statusMap,
        )
      : store.items,
  );

  function openEditModal(server: Server) {
    if (noc.isNocMode) return;
    isSortModalOpen = false;
    terminalServer = null;
    editingServer = server;
    isModalOpen = true;
  }

  function openSortModal() {
    if (noc.isNocMode) return;
    isModalOpen = false;
    terminalServer = null;
    isSortModalOpen = true;
  }

  function handleOpenTerminal(server: Server) {
    if (noc.isNocMode) return;
    isModalOpen = false;
    isSortModalOpen = false;
    terminalServer = server;
  }

  async function handleLaunchRemoteDesktop(server: Server) {
    if (noc.isNocMode) return;
    try {
      await invoke("launch_remote_desktop", {
        host: server.host,
        port: server.rdp_port ?? 3389,
        username: server.remote_username || null,
      });
    } catch (err: unknown) {
      store.errorMessage = `Failed to launch Remote Desktop: ${err}`;
    }
  }

  onMount(() => {
    store.init();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (terminalServer) {
          terminalServer = null;
        } else if (isModalOpen) {
          isModalOpen = false;
        } else if (isSortModalOpen) {
          isSortModalOpen = false;
        } else {
          noc.handleKeyDown(e);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      noc.cleanup();
    };
  });
</script>

<div
  class="page-container"
  class:noc-layout={noc.isNocMode}
  style:--card-min-width="{SERVER_CARD_MIN_WIDTH_REM * noc.cardScale}rem"
  style:--noc-min-width="{SERVER_NOC_MIN_WIDTH_REM * noc.cardScale}rem"
  style:--card-min-height="{SERVER_CARD_MIN_HEIGHT_REM * noc.cardScale}rem"
  style:--card-font-scale={noc.cardScale}
>
  <DashboardHeader
    title={noc.isNocMode ? "NOC Monitor" : "Servers"}
    countText="{displayServers.length} configured"
    isNocMode={noc.isNocMode}
    errorMessage={store.errorMessage}
    onAdjustScale={noc.adjustScale}
    onToggleNoc={() => noc.setNocMode(!noc.isNocMode)}
    onRetry={store.fetchServers}
  >
    {#snippet extraActions()}
      {#if clientMode.isRemoteClient && !clientMode.canEdit}
        <span class="readonly-badge" title="Connected to remote server in read-only mode">
          Remote (Read-Only)
        </span>
      {/if}
      {#if !noc.isNocMode}
        <button
          type="button"
          class="btn-sort"
          class:active={isSorted}
          onclick={openSortModal}
          title={isSorted ? `Sorting by ${sortRules.length} rule(s)` : "Configure sort rules"}
        >
          <Sort size={14} />
          <span>Sort</span>
          {#if isSorted}
            <Circle size={15} text={sortRules.length} class="sort-count-badge" />
          {/if}
        </button>
      {/if}
    {/snippet}
  </DashboardHeader>

  {#if store.loading}
    <p class="status-message">Reading configuration...</p>
  {:else}
    <section
      class="server-grid"
      class:noc-grid={noc.isNocMode}
      use:dndzone={{
        items: displayServers,
        flipDurationMs: DND_FLIP_DURATION_MS,
        dragDisabled: noc.isNocMode || isSorted || !clientMode.canEdit,
        dropTargetStyle: {},
      }}
      onconsider={(e) => {
        if (!isSorted && clientMode.canEdit) {
          store.items = e.detail.items;
        }
      }}
      onfinalize={(e) => {
        if (!isSorted && clientMode.canEdit) {
          store.reorderServers(e.detail.items);
        }
      }}
    >
      {#each displayServers as server (server.id)}
        {@const hostState = store.statusMap[server.id]}
        <div animate:flip={{ duration: DND_FLIP_DURATION_MS }} class="drag-wrapper">
          <ServerCard
            {server}
            nocMode={noc.isNocMode}
            status={hostState?.status ?? "checking"}
            latencyMs={hostState?.latencyMs}
            lastEvent={hostState?.lastEvent}
            lastEventAt={hostState?.lastEventAt}
            now={store.now}
            onclick={() => openEditModal(server)}
            onterminal={handleOpenTerminal}
            onremotedesktop={handleLaunchRemoteDesktop}
          />
        </div>
      {/each}

      {#if !noc.isNocMode && clientMode.canEdit}
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

<SortModal
  bind:open={isSortModalOpen}
  title="Sort Servers"
  bind:rules={sortRules}
  fields={SERVER_SORT_FIELDS}
/>

<ServerModal
  bind:open={isModalOpen}
  server={editingServer}
  onsave={(saved, id) => store.saveServer(saved, id)}
  ondelete={(id) => store.deleteServer(id)}
/>

{#if terminalServer}
  <ServerTerminalDrawer
    server={terminalServer}
    onclose={() => (terminalServer = null)}
  />
{/if}

<style>
  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--card-min-width, 18rem)), 1fr));
    gap: 1rem;
    width: 100%;
    outline: none;
  }

  .server-grid.noc-grid {
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--noc-min-width, 24rem)), 1fr));
  }

  .drag-wrapper {
    display: contents;
  }

  .add-card-wrapper {
    display: flex;
    min-height: var(--card-min-height, 8.5rem);
    width: 100%;
  }

  .add-server-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    height: 100%;
    min-height: var(--card-min-height, 8.5rem);
    background-color: transparent;
    border: dashed var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: border-color 0.15s ease, color 0.15s ease, background-color 0.15s ease;
  }

  .add-server-card:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background-color: color-mix(in srgb, var(--color-primary) 5%, transparent);
  }

  .plus-icon {
    font-size: 1.5rem;
    line-height: 1;
    font-weight: 300;
  }

  .add-title {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .status-message {
    color: var(--color-text-muted);
    font-size: 0.9rem;
    margin: 2rem 0;
    text-align: center;
  }

  .readonly-badge {
    display: inline-flex;
    align-items: center;
    height: 1.85rem;
    box-sizing: border-box;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0 0.5rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-warning) 15%, var(--color-surface));
    color: var(--color-warning);
    border: solid var(--width-border) color-mix(in srgb, var(--color-warning) 30%, transparent);
  }
</style>
