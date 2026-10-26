<script lang="ts">
  import DashboardHeader from "$lib/components/dashboard-header.svelte";
  import SortModal from "$lib/components/sort-modal.svelte";
  import {
    DND_FLIP_DURATION_MS,
    SERVER_CARD_MIN_HEIGHT_REM,
    SERVER_CARD_MIN_WIDTH_REM,
    SERVER_NOC_MIN_WIDTH_REM,
  } from "$lib/constants";
  import { Sort } from "$lib/icons";
  import { createNocController } from "$lib/noc.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import { loadSavedSortRules, saveSortRules, sortEntities } from "$lib/sort/sort";
  import type { SortRule } from "$lib/sort/types";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import ServerCard from "./server-card.svelte";
  import ServerModal from "./server-modal.svelte";
  import { serverStore } from "./servers.svelte";
  import { SERVER_SORT_FIELDS, SERVER_SORT_STORAGE_KEY } from "./sort";
  import type { Server } from "./types";

  const store = serverStore;
  const noc = createNocController(() => {
    isModalOpen = false;
    isSortModalOpen = false;
  });

  let isModalOpen = $state(false);
  let isSortModalOpen = $state(false);
  let editingServer = $state<Server | undefined>(undefined);
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
    editingServer = server;
    isModalOpen = true;
  }

  function openSortModal() {
    if (noc.isNocMode) return;
    isModalOpen = false;
    isSortModalOpen = true;
  }

  onMount(() => {
    store.init();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (isModalOpen) {
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
            <span class="count-badge sort-count-badge">{sortRules.length}</span>
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
            onclick={() => {
              if (clientMode.canEdit) {
                openEditModal(server);
              }
            }}
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

  .readonly-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.625rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-muted);
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 4px;
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
