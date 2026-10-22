<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onDestroy, onMount, tick } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import PrinterCard from "./printer-card.svelte";
  import PrinterModal from "./printer-modal.svelte";
  import { PrinterStore } from "./printers.svelte";
  import type { Printer } from "./types";

  const appWindow = getCurrentWindow();
  const store = new PrinterStore();

  let isModalOpen = $state(false);
  let editingPrinter = $state<Printer | undefined>(undefined);
  let isNocMode = $state(false);
  let cardScale = $state(1);

  let activeWebPrinter = $state<Printer | null>(null);
  let webviewUrl = $state("");
  let webviewError = $state<string | null>(null);
  let webviewSlotEl = $state<HTMLElement | undefined>(undefined);
  let resizeObserver: ResizeObserver | null = null;

  function adjustScale(delta: number) {
    cardScale = Math.min(1.6, Math.max(0.7, +(cardScale + delta).toFixed(2)));
  }

  async function setNocMode(active: boolean) {
    if (active && activeWebPrinter) {
      await closeWebInterface();
    }
    isNocMode = active;
    try {
      await appWindow.setFullscreen(active);
    } catch (err) {
      console.error("Failed to set fullscreen:", err);
    }
  }

  function resolvePrinterUrl(printer: Printer): string {
    const raw = printer.web_url?.trim() || printer.host.trim();
    if (raw.startsWith("http://") || raw.startsWith("https://")) {
      return raw;
    }
    return `http://${raw}`;
  }

  async function syncChildWebviewBounds() {
    if (!activeWebPrinter || !webviewSlotEl) return;
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

  async function openWebInterface(printer: Printer) {
    if (isNocMode) return;
    webviewError = null;
    activeWebPrinter = printer;
    webviewUrl = resolvePrinterUrl(printer);

    await tick();

    if (!webviewSlotEl) return;
    const rect = webviewSlotEl.getBoundingClientRect();

    try {
      const resolved = await invoke<string>("open_printer_webview", {
        url: webviewUrl,
        x: Math.round(rect.left),
        y: Math.round(rect.top),
        width: Math.max(200, Math.round(rect.width)),
        height: Math.max(200, Math.round(rect.height)),
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

  async function closeWebInterface() {
    resizeObserver?.disconnect();
    resizeObserver = null;
    activeWebPrinter = null;
    webviewError = null;
    try {
      await invoke("close_printer_webview");
    } catch (err) {
      console.error("Failed to close child webview:", err);
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
    if (!activeWebPrinter) return;
    const cleanHost = (activeWebPrinter.web_url?.trim() || activeWebPrinter.host.trim())
      .replace(/^https?:\/\//i, "");
    const nextUrl = `${scheme}://${cleanHost}`;
    try {
      webviewError = null;
      webviewUrl = await invoke<string>("navigate_printer_webview", { url: nextUrl });
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

  async function openEditModal(printer?: Printer) {
    if (isNocMode) return;
    if (activeWebPrinter) {
      await closeWebInterface();
    }
    editingPrinter = printer;
    isModalOpen = true;
  }

  onMount(() => {
    store.init();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (activeWebPrinter) {
          closeWebInterface();
        } else if (isNocMode) {
          setNocMode(false);
        }
      }
    };

    const handleWindowResize = () => {
      if (activeWebPrinter) {
        syncChildWebviewBounds();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", handleWindowResize);

    return () => {
      store.destroy();
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", handleWindowResize);
      appWindow.setFullscreen(false).catch(() => {});
    };
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    invoke("close_printer_webview").catch(() => {});
  });
</script>

{#if activeWebPrinter}
  <div class="webview-shell">
    <header class="webview-toolbar">
      <div class="toolbar-left">
        <button
          type="button"
          class="btn-back"
          onclick={closeWebInterface}
        >
          ← Back to Printers
        </button>

        <div class="printer-identity">
          <span
            class="status-dot {store.statusMap[activeWebPrinter.id]?.status ?? 'checking'}"
            aria-hidden="true"
          ></span>
          <strong class="toolbar-label">{activeWebPrinter.label}</strong>
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
          onclick={() => openEditModal(activeWebPrinter!)}
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
        <button type="button" class="btn-retry" onclick={() => openWebInterface(activeWebPrinter!)}>
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
{:else}
  <div
    class="page-container"
    class:noc-layout={isNocMode}
    style:--card-min-height="{15.5 * cardScale}rem"
    style:--card-font-scale={cardScale}
  >
    <header class="page-header">
      <div class="header-titles">
        <h1 class="page-title">{isNocMode ? "Printer NOC Monitor" : "Printers Dashboard"}</h1>
        <span class="printer-count">{store.items.length} configured</span>
      </div>
      <div class="header-actions">
        <div class="zoom-controls">
          <button
            type="button"
            class="btn-zoom"
            onclick={() => adjustScale(-0.1)}
            aria-label="Decrease card size"
          >
            −
          </button>
          <button
            type="button"
            class="btn-zoom"
            onclick={() => adjustScale(0.1)}
            aria-label="Increase card size"
          >
            +
          </button>
        </div>

        <button
          type="button"
          class="btn-noc"
          onclick={() => setNocMode(!isNocMode)}
        >
          {isNocMode ? "Exit NOC (Esc)" : "NOC Mode"}
        </button>
      </div>
    </header>

    {#if store.errorMessage}
      <div class="error-banner">
        <span>{store.errorMessage}</span>
        <button type="button" class="btn-retry" onclick={store.fetchPrinters}>Retry</button>
      </div>
    {/if}

    {#if store.loading}
      <p class="status-message">Reading printer configuration...</p>
    {:else}
      <section
        class="printer-grid"
        class:noc-grid={isNocMode}
        use:dndzone={{
          items: store.items,
          flipDurationMs: 200,
          dragDisabled: isNocMode,
          dropTargetStyle: {}
        }}
        onconsider={(e) => (store.items = e.detail.items)}
        onfinalize={(e) => store.reorderPrinters(e.detail.items)}
      >
        {#each store.items as printer (printer.id)}
          {@const printerState = store.statusMap[printer.id]}
          <div animate:flip={{ duration: 200 }} class="drag-wrapper">
            <PrinterCard
              {printer}
              nocMode={isNocMode}
              status={printerState?.status ?? "checking"}
              latencyMs={printerState?.latencyMs}
              lastEvent={printerState?.lastEvent}
              lastEventAt={printerState?.lastEventAt}
              toners={printerState?.toners ?? []}
              detectedModel={printerState?.detectedModel}
              serialNumber={printerState?.serialNumber}
              pageCount={printerState?.pageCount}
              printerState={printerState?.printerState}
              statusMessage={printerState?.statusMessage}
              now={store.now}
              onclick={() => openWebInterface(printer)}
              onconfigure={() => openEditModal(printer)}
            />
          </div>
        {/each}

        {#if !isNocMode}
          <div class="add-card-wrapper" aria-disabled="true">
            <button
              class="add-printer-card"
              type="button"
              onclick={() => openEditModal(undefined)}
              aria-label="Add new printer"
            >
              <span class="plus-icon" aria-hidden="true">+</span>
              <span class="add-title">Add printer</span>
              <span class="add-subtitle">Monitor connection, toner levels & web console</span>
            </button>
          </div>
        {/if}
      </section>
    {/if}
  </div>
{/if}

<PrinterModal
  bind:open={isModalOpen}
  printer={editingPrinter}
  onsave={(saved, id) => store.savePrinter(saved, id)}
  ondelete={(id) => store.deletePrinter(id)}
/>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.5rem;
  }

  .page-container.noc-layout {
    position: fixed;
    inset: 0;
    box-sizing: border-box;
    background-color: var(--color-base);
    padding: 2rem 2.5rem;
    overflow-y: auto;
    z-index: 50;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .page-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .printer-count {
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .zoom-controls {
    display: flex;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    overflow: hidden;
  }

  .btn-zoom {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    padding: 0.35rem 0.65rem;
    font-size: 0.95rem;
    font-weight: 700;
    line-height: 1;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .btn-zoom:first-child {
    border-right: solid var(--width-border) var(--color-border);
  }

  .btn-zoom:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .btn-noc {
    background-color: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.35rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-noc:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  /* Printer cards take up half of their container's width (2 columns) */
  .printer-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1.25rem;
    width: 100%;
    outline: none;
  }

  @media (max-width: 900px) {
    .printer-grid {
      grid-template-columns: 1fr;
    }
  }

  .printer-grid.noc-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1.5rem;
  }

  .drag-wrapper,
  .add-card-wrapper {
    height: 100%;
    min-height: var(--card-min-height, 15.5rem);
    outline: none;
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-failure) 15%, transparent);
    color: var(--color-failure);
    border: solid var(--width-border) var(--color-failure);
  }

  .btn-retry {
    background: transparent;
    border: solid var(--width-border) var(--color-failure);
    color: var(--color-failure);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .add-printer-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.45rem;
    min-height: var(--card-min-height, 15.5rem);
    height: 100%;
    width: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 1.5rem;
    background-color: transparent;
    border: 2px dashed var(--color-border);
    border-radius: 0.5rem;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
  }

  .add-printer-card:hover {
    background-color: color-mix(in srgb, var(--color-surface) 30%, transparent);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .add-printer-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .plus-icon {
    font-size: calc(2.25rem * var(--card-font-scale, 1));
    font-weight: 300;
    line-height: 1;
  }

  .add-title {
    font-size: calc(1.05rem * var(--card-font-scale, 1));
    font-weight: 600;
  }

  .add-subtitle {
    font-size: calc(0.8rem * var(--card-font-scale, 1));
    opacity: 0.75;
  }

  .status-message {
    color: var(--color-text-muted);
    font-size: 0.95rem;
  }

  /* Child Webview Host Layout */
  .webview-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .webview-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    height: 48px;
    padding-inline: 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    min-width: 0;
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

  .status-dot {
    width: 0.55rem;
    height: 0.55rem;
    border-radius: 50%;
    background-color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .status-dot.online {
    background-color: var(--color-success);
  }

  .status-dot.degraded {
    background-color: var(--color-warning);
  }

  .status-dot.offline {
    background-color: var(--color-failure);
  }

  .toolbar-label {
    font-size: 0.95rem;
    color: var(--color-text);
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
