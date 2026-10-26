<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { endpointStore } from "./endpoint.svelte";
  import HardwarePanel from "./hardware-panel.svelte";
  import TerminalWindow from "./terminal-window.svelte";
  import UnsupportedOs from "./unsupported-os.svelte";
  import UpdatesWindow from "./updates-window.svelte";

  const store = endpointStore;

  onMount(() => {
    store.subscribe().catch((err) => {
      console.error("Endpoint initialization failed:", err);
    });
  });

  onDestroy(() => {
    store.unsubscribe();
  });
</script>

<svelte:head>
  <title>Caffeine - Endpoint</title>
</svelte:head>

<div class="endpoint-page-shell">
  {#if store.osChecking}
    <div class="page-loading-state">
      <div class="spinner"></div>
      <span>Verifying operating system compatibility...</span>
    </div>
  {:else if !store.osSupport?.is_windows}
    <UnsupportedOs osSupport={store.osSupport} />
  {:else}
    <div class="endpoint-workspace">
      <!-- Left Column: System & Hardware Telemetry -->
      <section class="left-column">
        <HardwarePanel
          machine={store.machine}
          isLoading={store.machineLoading}
          errorMessage={store.machineError}
          onRefresh={() => store.fetchMachineInfo(true)}
        />
      </section>

      <!-- Right Column: Two Stacked Windows -->
      <section class="right-column">
        <!-- Top Window: System Updates -->
        <div class="top-window-wrap">
          <UpdatesWindow
            updates={store.updates}
            isChecking={store.updatesChecking}
            downloadingId={store.downloadingId}
            installingId={store.installingId}
            operationStatus={store.operationStatus}
            errorMessage={store.updatesError}
            onCheck={() => store.checkForUpdates()}
            onDownload={(id) => store.downloadUpdates(id)}
            onInstall={(id) => store.installUpdates(id)}
          />
        </div>

        <!-- Bottom Window: Terminal -->
        <div class="bottom-window-wrap">
          <TerminalWindow shell={store.shell} />
        </div>
      </section>
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

  .endpoint-workspace {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
    padding: 0.85rem;
    gap: 0.85rem;
    box-sizing: border-box;
    background-color: var(--color-base);
  }

  .left-column {
    flex: 1.1;
    min-width: 340px;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .right-column {
    flex: 1;
    min-width: 340px;
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    gap: 0.85rem;
    box-sizing: border-box;
  }

  .top-window-wrap {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .bottom-window-wrap {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
