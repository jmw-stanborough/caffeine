<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import ServerCard from "./server-card.svelte";
  import ServerModal from "./server-modal.svelte";

  interface Agent {
    port: number;
    api_key: string;
  }

  interface Server {
    host: string;
    label: string;
    description?: string;
    agent?: Agent;
  }

  type DndServer = Server & { id: string };

  let items = $state<DndServer[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);

  let isModalOpen = $state(false);
  let editingServer = $state<Server | undefined>(undefined);

  async function fetchServers() {
    loading = true;
    errorMessage = null;
    try {
      const rawServers = await invoke<Server[]>("load_servers");
      items = rawServers.map((s) => ({ ...s, id: s.host }));
    } catch (err) {
      errorMessage = String(err);
    } finally {
      loading = false;
    }
  }

  function openAddModal() {
    editingServer = undefined;
    isModalOpen = true;
  }

  function openEditModal(server: Server) {
    editingServer = server;
    isModalOpen = true;
  }

  async function handleSaveServer(savedServer: Server, originalHost?: string) {
    try {
      let updated: Server[];
      if (originalHost) {
        updated = await invoke<Server[]>("update_server", {
          originalHost,
          updatedServer: savedServer
        });
      } else {
        updated = await invoke<Server[]>("save_server", {
          newServer: savedServer
        });
      }
      items = updated.map((s) => ({ ...s, id: s.host }));
    } catch (err) {
      errorMessage = String(err);
    }
  }

  function handleDndConsider(e: CustomEvent<{ items: DndServer[] }>) {
    items = e.detail.items;
  }

  async function handleDndFinalize(e: CustomEvent<{ items: DndServer[] }>) {
    items = e.detail.items;

    const payload = items.map(({ id, ...rest }) => rest);

    try {
      await invoke("reorder_servers", { servers: payload });
    } catch (err) {
      errorMessage = String(err);
    }
  }

  onMount(fetchServers);
</script>

<div class="page-container">
  <header class="page-header">
    <div class="header-titles">
      <h1 class="page-title">Servers</h1>
      <span class="server-count">{items.length} configured</span>
    </div>
  </header>

  {#if errorMessage}
    <div class="error-banner">
      <span>{errorMessage}</span>
      <button type="button" class="btn-retry" onclick={fetchServers}>Retry</button>
    </div>
  {/if}

  {#if loading}
    <p class="status-message">Reading configuration...</p>
  {:else}
    <section
      class="server-grid"
      use:dndzone={{ items, flipDurationMs: 200, dropTargetStyle: {} }}
      onconsider={handleDndConsider}
      onfinalize={handleDndFinalize}
    >
      {#each items as server (server.id)}
        <div animate:flip={{ duration: 200 }} class="drag-wrapper">
          <ServerCard {server} onclick={() => openEditModal(server)} />
        </div>
      {/each}

      <div class="add-card-wrapper" aria-disabled="true">
        <button
          class="add-server-card"
          type="button"
          onclick={openAddModal}
          aria-label="Add new server"
        >
          <span class="plus-icon" aria-hidden="true">+</span>
          <span class="add-title">Add server</span>
        </button>
      </div>
    </section>
  {/if}
</div>

<ServerModal
  bind:open={isModalOpen}
  server={editingServer}
  onsave={handleSaveServer}
/>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.5rem;
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

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-secondary) 15%, transparent);
    color: var(--color-secondary);
    border: solid var(--width-border) var(--color-secondary);
  }

  .btn-retry {
    background: transparent;
    border: solid var(--width-border) var(--color-secondary);
    color: var(--color-secondary);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    gap: 1rem;
    width: 100%;
    outline: none;
  }

  .drag-wrapper {
    height: 100%;
    outline: none;
  }

  .add-card-wrapper {
    height: 100%;
    outline: none;
  }

  .add-server-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    min-height: 8.5rem;
    height: 100%;
    width: 100%;
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
    font-size: 2rem;
    font-weight: 300;
    line-height: 1;
  }

  .add-title {
    font-size: 1rem;
    font-weight: 600;
  }

  .status-message {
    color: var(--color-text-muted);
    font-size: 0.95rem;
  }
</style>
