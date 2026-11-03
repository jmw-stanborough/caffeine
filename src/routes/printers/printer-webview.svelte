<script lang="ts">
  import { MIN_WEBVIEW_DIMENSION_PX } from "$lib/constants";
  import { Circle } from "$lib/icons";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onDestroy, onMount, tick } from "svelte";
  import { getEffectivePrinterStatus } from "./tags";
  import type { Printer, PrinterState } from "./types";

  let {
    printer,
    printerState = undefined,
    onclose,
    onconfigure,
  }: {
    printer: Printer;
    printerState?: PrinterState;
    onclose: () => void;
    onconfigure: (printer: Printer) => void;
  } = $props();

  let webviewUrl = $state("");
  let webviewError = $state<string | null>(null);
  let webviewSlotEl = $state<HTMLElement | undefined>(undefined);
  let resizeObserver: ResizeObserver | null = null;

  let effectiveStatus = $derived(getEffectivePrinterStatus(printerState));

  function resolvePrinterUrl(target: Printer): string {
    const raw = target.web_url?.trim() || target.host.trim();
    if (raw.startsWith("http://") || raw.startsWith("https://")) {
      return raw;
    }
    return `http://${raw}`;
  }

  async function syncChildWebviewBounds() {
    if (!webviewSlotEl) return;
    const rect = webviewSlotEl.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;

    try {
      await invoke("resize_printer_webview", {
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      });
    } catch (err) {
      console.error("Failed to resize child webview:", err);
    }
  }

  async function launchWebview() {
    webviewError = null;
    webviewUrl = resolvePrinterUrl(printer);

    await tick();

    if (!webviewSlotEl) return;
    const rect = webviewSlotEl.getBoundingClientRect();

    try {
      const resolved = await invoke<string>("open_printer_webview", {
        url: webviewUrl,
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.max(MIN_WEBVIEW_DIMENSION_PX, Math.round(rect.width)),
        height: Math.max(MIN_WEBVIEW_DIMENSION_PX, Math.round(rect.height)),
      });
      webviewUrl = resolved;

      resizeObserver?.disconnect();
      resizeObserver = new ResizeObserver(() => {
        syncChildWebviewBounds();
      });
      resizeObserver.observe(webviewSlotEl);
    } catch (err) {
      webviewError = String(err);
    }
  }

  async function reloadWebInterface() {
    try {
      await invoke("reload_printer_webview");
    } catch (err) {
      webviewError = String(err);
    }
  }

  async function switchScheme(scheme: "http" | "https") {
    const cleanHost = (printer.web_url?.trim() || printer.host.trim()).replace(
      /^https?:\/\//i,
      "",
    );
    const nextUrl = `${scheme}://${cleanHost}`;
    try {
      webviewError = null;
      webviewUrl = await invoke<string>("navigate_printer_webview", {
        url: nextUrl,
      });
    } catch (err) {
      webviewError = String(err);
    }
  }

  async function openExternalBrowser() {
    if (!webviewUrl) return;
    try {
      await openUrl(webviewUrl);
    } catch (err) {
      console.error("Failed to open external browser:", err);
    }
  }

  onMount(() => {
    launchWebview();
    window.addEventListener("resize", syncChildWebviewBounds);
    return () => {
      window.removeEventListener("resize", syncChildWebviewBounds);
    };
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    resizeObserver = null;
    invoke("close_printer_webview").catch((err) => {
      console.error("Failed to close printer webview on destroy:", err);
    });
  });
</script>

<div class="webview-shell">
  <header class="webview-toolbar">
    <div class="toolbar-left">
      <button type="button" class="btn-back" onclick={onclose}>
        ← Back to Printers
      </button>

      <div class="printer-identity">
        <Circle size={10} class="status-dot {effectiveStatus}" aria-hidden="true" />
        <strong class="toolbar-label">{printer.label}</strong>
        <span class="toolbar-url selectable">{webviewUrl}</span>
      </div>
    </div>

    <div class="toolbar-actions">
      <div class="scheme-switcher">
        <button
          type="button"
          class="btn-tool"
          class:active={webviewUrl.startsWith("http://")}
          onclick={() => switchScheme("http")}
        >
          HTTP
        </button>
        <button
          type="button"
          class="btn-tool"
          class:active={webviewUrl.startsWith("https://")}
          onclick={() => switchScheme("https")}
        >
          HTTPS
        </button>
      </div>

      <button
        type="button"
        class="btn-tool"
        onclick={reloadWebInterface}
        title="Reload printer web interface"
      >
        ↻ Reload
      </button>

      <button
        type="button"
        class="btn-tool"
        onclick={() => onconfigure(printer)}
        title="Configure this printer"
      >
        ⚙ Settings
      </button>

      <button
        type="button"
        class="btn-tool"
        onclick={openExternalBrowser}
        title="Open in system browser"
      >
        External ↗
      </button>
    </div>
  </header>

  {#if webviewError}
    <div class="error-banner webview-error">
      <span>Failed to load child webview: {webviewError}</span>
      <button type="button" class="btn-retry" onclick={launchWebview}>
        Retry
      </button>
    </div>
  {/if}

  <div class="webview-host-slot" bind:this={webviewSlotEl}>
    <div class="webview-loading-backdrop">
      <span>Loading native child webview for {webviewUrl}...</span>
    </div>
  </div>
</div>

<style>
  .webview-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .webview-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem 1rem;
    min-height: 48px;
    padding: 0.4rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    min-width: 0;
    flex: 1;
  }

  .btn-back {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.35rem 0.7rem;
    border-radius: 0.35rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: border-color 0.15s ease, color 0.15s ease;
  }

  .btn-back:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .printer-identity {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .toolbar-label {
    font-size: 0.95rem;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toolbar-url {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--color-text-muted);
    background-color: var(--color-base);
    padding: 0.2rem 0.5rem;
    border-radius: 0.25rem;
    border: solid var(--width-border) var(--color-border);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toolbar-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.45rem;
    flex-shrink: 0;
  }

  .scheme-switcher {
    display: flex;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
    overflow: hidden;
  }

  .scheme-switcher .btn-tool {
    border: none;
    border-radius: 0;
  }

  .scheme-switcher .btn-tool:first-child {
    border-right: solid var(--width-border) var(--color-border);
  }

  .btn-tool {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.3rem 0.6rem;
    border-radius: 0.35rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .btn-tool:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .btn-tool.active {
    background-color: var(--color-primary);
    color: var(--color-base);
  }

  .webview-error {
    margin: 0.5rem 1rem 0;
  }

  .webview-host-slot {
    flex: 1;
    width: 100%;
    min-height: 0;
    position: relative;
    background-color: var(--color-base);
  }

  .webview-loading-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }
</style>
