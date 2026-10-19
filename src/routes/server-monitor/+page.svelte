<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ServerCard from "./server-card.svelte";
  import AddServerModal from "./add-server-modal.svelte";

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

  let servers = $state<Server[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
  let isModalOpen = $state(false);

  async function fetchServers() {
    loading = true;
    errorMessage = null;
    try {
      servers = await invoke<Server[]>("load_servers");
    } catch (err) {
      errorMessage = String(err);
    } finally {
      loading = false;
    }
  }

  async function handleSaveServer(newServer: Server) {
    try {
      servers = await invoke<Server[]>("save_server", { newServer });
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
      <span class="server-count">{servers.length} configured</span>
    </div>

    <button class="add-button" type="button" onclick={() => (isModalOpen = true)}>
      Add Server
    </button>
  </header>

  {#if errorMessage}
    <div class="error-banner">
      <span>{errorMessage}</span>
      <button type="button" onclick={fetchServers}>Retry</button>
    </div>
  {/if}

  {#if loading}
    <p class="status-message">Reading configuration...</p>
  {:else if servers.length === 0}
    <p class="status-message">No servers configured. Click "Add Server" to register one.</p>
  {:else}
    <section class="server-grid">
      {#each servers as server (server.host)}
        <ServerCard {server} />
      {/each}
    </section>
  {/if}
</div>

<AddServerModal bind:open={isModalOpen} onsave={handleSaveServer} />

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

  .add-button {
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    font-weight: 600;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    background-color: var(--color-surface);
    color: var(--color-primary);
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .add-button:hover {
    background-color: var(--color-surface-hover);
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-secondary) 15%, transparent);
    color: var(--color-secondary);
    border: solid var(--width-border) var(--color-secondary);
  }

  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
    gap: 1rem;
    width: 100%;
  }

  .status-message {
    color: var(--color-text-muted);
    font-size: 0.95rem;
  }
</style>
