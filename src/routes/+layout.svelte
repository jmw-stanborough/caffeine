<script lang="ts">
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Sidebar from "$lib/sidebar.svelte";
  import { serverStore } from "./servers/servers.svelte";
  import { printerStore } from "./printers/printers.svelte";
  import { applyTheme } from "$lib/theme";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(() => {
    let unlistenSync: UnlistenFn | null = null;

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
    {@render children()}
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
</style>
