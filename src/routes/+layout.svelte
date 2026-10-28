<script lang="ts">
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Sidebar from "$lib/sidebar.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import { serverStore } from "./servers/servers.svelte";
  import { printerStore } from "./printers/printers.svelte";
  import { endpointStore } from "./endpoint/endpoint.svelte";
  import EndpointModuleShell from "./endpoint/endpoint-module-shell.svelte";
  import TerminalWindow from "./endpoint/terminal-window.svelte";
  import { applyTheme } from "$lib/theme";
  import { page } from "$app/state";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let { children } = $props();

  let isTerminalRoute = $derived(page.url.pathname === "/terminal");
  let hasVisitedTerminal = $state(page.url.pathname === "/terminal");

  $effect(() => {
    if (isTerminalRoute && !hasVisitedTerminal) {
      hasVisitedTerminal = true;
    }
  });

  onMount(() => {
    let unlistenSync: UnlistenFn | null = null;
    clientMode.refresh();

    invoke<{ theme?: string }>("get_app_settings_cmd")
      .then((settings) => {
        if (settings?.theme) {
          applyTheme(settings.theme);
        }
      })
      .catch((err) => {
        console.warn("Failed to load user theme setting:", err);
      });

    serverStore.init().catch((err) => {
      console.error("Failed to initialize server monitor on startup:", err);
    });
    printerStore.init().catch((err) => {
      console.error("Failed to initialize printer monitor on startup:", err);
    });

    listen("snapshot-synced", () => {
      serverStore.fetchServers();
      printerStore.fetchPrinters();
    }).then((unlisten) => {
      unlistenSync = unlisten;
    }).catch((err) => {
      console.error("Failed to listen for snapshot sync events:", err);
    });

    return () => {
      unlistenSync?.();
    };
  });
</script>

<div class="app-shell">
  <Sidebar />
  <main class="content-area">
    <div class="view-wrapper" class:hidden={isTerminalRoute}>
      {@render children()}
    </div>
    {#if hasVisitedTerminal}
      <div class="terminal-persistent-wrapper" class:hidden={!isTerminalRoute}>
        <EndpointModuleShell title="Terminal">
          <div class="module-fill">
            <TerminalWindow shell={endpointStore.shell} active={isTerminalRoute} />
          </div>
        </EndpointModuleShell>
      </div>
    {/if}
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .content-area {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    background-color: var(--color-base);
  }

  .view-wrapper,
  .terminal-persistent-wrapper {
    width: 100%;
    height: 100%;
  }

  .module-fill {
    flex: 1;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .hidden {
    display: none !important;
  }
</style>
