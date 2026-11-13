<script lang="ts">
  import type { Snippet } from "svelte";
  import { onDestroy, onMount } from "svelte";
  import { endpointStore } from "./endpoint.svelte";
  import UnsupportedOs from "./unsupported-os.svelte";

  interface Props {
    title: string;
    enableTelemetry?: boolean;
    children: Snippet;
  }

  let { title, enableTelemetry = false, children }: Props = $props();
  const store = endpointStore;

  onMount(() => {
    if (enableTelemetry) {
      store.subscribe().catch((err) => {
        console.error("Endpoint initialization failed:", err);
      });
    } else {
      store.init().catch((err) => {
        console.error("Endpoint initialization failed:", err);
      });
    }
  });

  onDestroy(() => {
    if (enableTelemetry) {
      store.unsubscribe();
    }
  });
</script>

<svelte:head>
  <title>Caffeine - {title}</title>
</svelte:head>

<div class="endpoint-page-shell">
  {#if store.osChecking}
    <div class="page-loading-state">
      <div class="spinner"></div>
      <span>Verifying operating system compatibility...</span>
    </div>
  {:else if !store.isOsSupported}
    <UnsupportedOs osSupport={store.osSupport} />
  {:else}
    <div class="endpoint-page-content">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .endpoint-page-shell {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-base);
  }

  .page-loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 0.95rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-surface);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .endpoint-page-content {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    padding: var(--page-padding);
    box-sizing: border-box;
    position: relative;
  }
</style>
