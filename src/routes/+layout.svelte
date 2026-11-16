<script lang="ts">
  import "@fontsource-variable/roboto";
  import "../app.css";
  import ReminderBanner from "$lib/components/reminder-banner.svelte";
  import Sidebar from "$lib/sidebar.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import {
    getPersistentViews,
    handleSnapshotSync,
    initializeBackgroundFeatures,
  } from "$lib/features";
  import { applyTheme } from "$lib/theme";
  import { soundManager, type SoundSettings } from "$lib/sound.svelte";
  import { page } from "$app/state";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let { children } = $props();

  const persistentViews = getPersistentViews();
  let isCurrentRoutePersistent = $derived(
    persistentViews.some((v) => page.url.pathname === v.href),
  );

  onMount(() => {
    let unlistenSync: UnlistenFn | null = null;
    clientMode.refresh();

    invoke<{ theme?: string } & SoundSettings>("get_app_settings_cmd")
      .then((settings) => {
        if (settings?.theme) {
          applyTheme(settings.theme);
        }
        return soundManager.refreshFromSettings(settings);
      })
      .catch((err) => {
        console.warn("Failed to load user theme or sound settings:", err);
      });

    initializeBackgroundFeatures().catch((err) => {
      console.error("Failed to initialize background features on startup:", err);
    });

    listen("snapshot-synced", () => {
      handleSnapshotSync().catch((err) => {
        console.error("Failed to handle snapshot sync:", err);
      });
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
    <ReminderBanner />
    <div class="view-wrapper" class:hidden={isCurrentRoutePersistent}>
      {@render children()}
    </div>
    {#each persistentViews as view (view.id)}
      {@const isActive = page.url.pathname === view.href}
      <div class="persistent-view-wrapper" class:hidden={!isActive}>
        <view.component active={isActive} />
      </div>
    {/each}
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
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow: hidden;
    background-color: var(--color-base);
  }

  .view-wrapper,
  .persistent-view-wrapper {
    width: 100%;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .hidden {
    display: none !important;
  }
</style>

