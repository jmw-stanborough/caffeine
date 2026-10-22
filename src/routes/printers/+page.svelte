<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onDestroy, onMount, tick } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import PrinterCard from "./printer-card.svelte";
  import PrinterDetailModal from "./printer-detail-modal.svelte";
  import PrinterModal from "./printer-modal.svelte";
  import { PrinterStore } from "./printers.svelte";
  import { filterPrintersByTags, isWarningTag } from "./tags";
  import type { Printer } from "./types";

  const appWindow = getCurrentWindow();
  const store = new PrinterStore();

  let isModalOpen = $state(false);
  let editingPrinter = $state<Printer | undefined>(undefined);
  let isDetailModalOpen = $state(false);
  let detailPrinterId = $state<string | null>(null);
  let isNocMode = $state(false);
  let cardScale = $state(1);

  let filterInputText = $state("");
  let filterInputEl = $state<HTMLInputElement | undefined>(undefined);
  let isFilterFocused = $state(false);

  let isFiltered = $derived(store.filterTags.length > 0);
  let filteredPrinters = $derived(
    filterPrintersByTags(store.items, store.statusMap, store.filterTags)
  );
  let premadeTags = $derived(store.getPremadeTagSuggestions());

  let detailPrinter = $derived(
    detailPrinterId ? store.items.find((p) => p.id === detailPrinterId) : undefined
  );
  let detailPrinterState = $derived(
    detailPrinterId ? store.statusMap[detailPrinterId] : undefined
  );

  let activeWebPrinter = $state<Printer | null>(null);
  let webviewUrl = $state("");
  let webviewError = $state<string | null>(null);
  let webviewSlotEl = $state<HTMLElement | undefined>(undefined);
  let resizeObserver: ResizeObserver | null = null;

  function adjustScale(delta: number) {
    cardScale = Math.min(1.6, Math.max(0.7, +(cardScale + delta).toFixed(2)));
  }

  async function handleFilterKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      const trimmed = filterInputText.trim();
      if (trimmed) {
        await store.addFilterTag(trimmed);
        filterInputText = "";
      }
    } else if (e.key === "Backspace" && filterInputText === "" && store.filterTags.length > 0) {
      e.preventDefault();
      await store.removeLastFilterTag();
    }
  }

  async function handlePremadeTagClick(tag: string) {
    await store.toggleFilterTag(tag);
    filterInputEl?.focus();
  }

  async function setNocMode(active: boolean) {
    if (active) {
      isDetailModalOpen = false;
      isModalOpen = false;
      if (activeWebPrinter) {
        await closeWebInterface();
      }
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
    isDetailModalOpen = false;
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

  function openDetailModal(printer: Printer) {
    if (isNocMode) return;
    detailPrinterId = printer.id;
    isDetailModalOpen = true;
  }

  async function openEditModal(printer?: Printer) {
    if (isNocMode) return;
    isDetailModalOpen = false;
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
        if (isModalOpen) {
          isModalOpen = false;
        } else if (isDetailModalOpen) {
          isDetailModalOpen = false;
        } else if (activeWebPrinter) {
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
    style:--card-min-width="{24 * cardScale}rem"
    style:--noc-min-width="{20 * cardScale}rem"
    style:--card-font-scale={cardScale}
  >
    <header class="page-header">
      <div class="header-titles">
        <h1 class="page-title">{isNocMode ? "Printer NOC Monitor" : "Printers Dashboard"}</h1>
        <span class="printer-count">
          {isFiltered
            ? `${filteredPrinters.length} of ${store.items.length} shown`
            : `${store.items.length} configured`}
        </span>
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

    {#if !isNocMode}
      <div class="filter-section">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="tag-entry-box"
          class:focused={isFilterFocused}
          onclick={() => filterInputEl?.focus()}
        >
          {#each store.filterTags as tag (tag)}
            <span
              class="printer-tag entry-tag"
              class:tag-warning={isWarningTag(tag)}
              class:tag-standard={!isWarningTag(tag)}
            >
              <span class="entry-tag-label">{tag}</span>
              <button
                type="button"
                class="entry-tag-remove"
                onclick={(e) => {
                  e.stopPropagation();
                  store.removeFilterTag(tag);
                  filterInputEl?.focus();
                }}
                aria-label="Remove {tag} filter"
                title="Remove {tag}"
              >
                ×
              </button>
            </span>
          {/each}

          <input
            type="text"
            class="tag-entry-input"
            bind:this={filterInputEl}
            bind:value={filterInputText}
            onfocus={() => (isFilterFocused = true)}
            onblur={() => (isFilterFocused = false)}
            onkeydown={handleFilterKeyDown}
            placeholder={store.filterTags.length === 0
              ? "Filter by tag (type a tag and press Enter)..."
              : "Add tag..."}
            aria-label="Filter printers by tag"
          />

          {#if store.filterTags.length > 0}
            <button
              type="button"
              class="btn-clear-tags"
              onclick={(e) => {
                e.stopPropagation();
                store.clearFilterTags();
                filterInputText = "";
                filterInputEl?.focus();
              }}
              title="Clear all filter tags"
            >
              Clear
            </button>
          {/if}
        </div>

        <div class="premade-tags-bar">
          <span class="premade-tags-label">Tags:</span>
          <div class="premade-tags-list">
            {#each premadeTags as tag (tag)}
              {@const active = store.hasFilterTag(tag)}
              <button
                type="button"
                class="printer-tag premade-tag-btn"
                class:tag-warning={isWarningTag(tag)}
                class:tag-standard={!isWarningTag(tag)}
                class:active
                onclick={() => handlePremadeTagClick(tag)}
                aria-pressed={active}
                title={active ? `Remove "${tag}" filter` : `Add "${tag}" filter`}
              >
                <span>{tag}</span>
                <span class="premade-tag-indicator" aria-hidden="true">
                  {active ? "✓" : "+"}
                </span>
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    {#if store.errorMessage}
      <div class="error-banner">
        <span>{store.errorMessage}</span>
        <button type="button" class="btn-retry" onclick={store.fetchPrinters}>Retry</button>
      </div>
    {/if}

    {#if store.loading}
      <p class="status-message">Reading printer configuration...</p>
    {:else}
      {#if isFiltered && store.items.length > 0 && filteredPrinters.length === 0}
        <div class="empty-filter-banner">
          <span>No printers match the active filter tags.</span>
          <button
            type="button"
            class="btn-clear-empty"
            onclick={() => store.clearFilterTags()}
          >
            Clear Filters
          </button>
        </div>
      {/if}

      <section
        class="printer-grid"
        class:noc-grid={isNocMode}
        use:dndzone={{
          items: filteredPrinters,
          flipDurationMs: 200,
          dragDisabled: isNocMode || isFiltered,
          dropTargetStyle: {}
        }}
        onconsider={(e) => {
          if (!isFiltered) {
            store.items = e.detail.items;
          }
        }}
        onfinalize={(e) => {
          if (!isFiltered) {
            store.reorderPrinters(e.detail.items);
          }
        }}
      >
        {#each filteredPrinters as printer (printer.id)}
          {@const printerState = store.statusMap[printer.id]}
          <div animate:flip={{ duration: 200 }} class="drag-wrapper">
            <PrinterCard
              {printer}
              nocMode={isNocMode}
              status={printerState?.status ?? "checking"}
              toners={printerState?.toners ?? []}
              detectedModel={printerState?.detectedModel}
              printerState={printerState?.printerState}
              onclick={() => openDetailModal(printer)}
              onwebui={() => openWebInterface(printer)}
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

<PrinterDetailModal
  bind:open={isDetailModalOpen}
  printer={detailPrinter}
  status={detailPrinterState?.status ?? "checking"}
  latencyMs={detailPrinterState?.latencyMs}
  lastEvent={detailPrinterState?.lastEvent}
  lastEventAt={detailPrinterState?.lastEventAt}
  toners={detailPrinterState?.toners ?? []}
  detectedModel={detailPrinterState?.detectedModel}
  serialNumber={detailPrinterState?.serialNumber}
  pageCount={detailPrinterState?.pageCount}
  printerState={detailPrinterState?.printerState}
  statusMessage={detailPrinterState?.statusMessage}
  now={store.now}
  onwebui={(p) => openWebInterface(p)}
  onconfigure={(p) => openEditModal(p)}
/>

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
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem 1rem;
  }

  .header-titles {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem 0.75rem;
    min-width: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .page-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text);
    white-space: nowrap;
  }

  .printer-count {
    font-size: 0.9rem;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .zoom-controls {
    display: flex;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    overflow: hidden;
    flex-shrink: 0;
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
    white-space: nowrap;
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
    white-space: nowrap;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }

  .btn-noc:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  /* Tag Filter Section */
  .filter-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .tag-entry-box {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    min-height: 2.5rem;
    padding: 0.35rem 0.65rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.45rem;
    cursor: text;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .tag-entry-box.focused {
    border-color: var(--color-primary);
  }

  .printer-tag {
    display: inline-flex;
    align-items: center;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.22rem 0.5rem;
    border-radius: 0.25rem;
    line-height: 1;
    border: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
    white-space: nowrap;
    box-sizing: border-box;
  }

  .tag-standard {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
  }

  .tag-warning {
    background-color: var(--color-warning);
    color: #1c1917;
    border-color: color-mix(in srgb, var(--color-warning) 75%, #000000);
  }

  .entry-tag {
    gap: 0.3rem;
    padding-right: 0.3rem;
  }

  .entry-tag-label {
    line-height: 1;
  }

  .entry-tag-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: inherit;
    font-size: 0.85rem;
    font-weight: 700;
    line-height: 1;
    padding: 0.05rem 0.18rem;
    border-radius: 0.2rem;
    cursor: pointer;
    opacity: 0.7;
    transition: opacity 0.12s ease, background-color 0.12s ease;
  }

  .entry-tag-remove:hover {
    opacity: 1;
    background-color: rgba(0, 0, 0, 0.12);
  }

  .tag-entry-input {
    flex: 1;
    min-width: 12rem;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 0.88rem;
    font-family: inherit;
    padding: 0.15rem 0.1rem;
    outline: none;
  }

  .tag-entry-input::placeholder {
    color: var(--color-text-muted);
    opacity: 0.8;
  }

  .btn-clear-tags {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    cursor: pointer;
    margin-left: auto;
    transition: color 0.15s ease, background-color 0.15s ease;
  }

  .btn-clear-tags:hover {
    color: var(--color-text);
    background-color: color-mix(in srgb, var(--color-base) 55%, transparent);
  }

  .premade-tags-bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .premade-tags-label {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .premade-tags-list {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .premade-tag-btn {
    gap: 0.3rem;
    cursor: pointer;
    opacity: 0.85;
    transition: opacity 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
  }

  .premade-tag-btn:hover {
    opacity: 1;
    border-color: var(--color-primary);
  }

  .premade-tag-btn.active {
    opacity: 1;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary);
  }

  .premade-tag-indicator {
    font-size: 0.72rem;
    font-weight: 700;
    opacity: 0.75;
  }

  .empty-filter-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 0.45rem;
    background-color: color-mix(in srgb, var(--color-surface) 65%, transparent);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    font-size: 0.88rem;
  }

  .btn-clear-empty {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.3rem 0.65rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: border-color 0.15s ease, color 0.15s ease;
  }

  .btn-clear-empty:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  /* Auto-fill responsive layout: cards take up strictly the space needed while looking uniform */
  .printer-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--card-min-width, 24rem)), 1fr));
    grid-auto-rows: auto;
    align-items: stretch;
    gap: 1rem;
    width: 100%;
    outline: none;
  }

  .printer-grid.noc-grid {
    grid-template-columns: repeat(auto-fill, minmax(min(100%, var(--noc-min-width, 20rem)), 1fr));
    grid-auto-rows: auto;
    align-items: stretch;
    gap: 1.25rem;
  }

  .drag-wrapper,
  .add-card-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
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
    gap: 0.35rem;
    min-height: 5rem;
    height: 100%;
    width: 100%;
    box-sizing: border-box;
    margin: 0;
    padding: 0.85rem 1.1rem;
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
    font-size: calc(1.5rem * var(--card-font-scale, 1));
    font-weight: 300;
    line-height: 1;
  }

  .add-title {
    font-size: calc(0.95rem * var(--card-font-scale, 1));
    font-weight: 600;
  }

  .add-subtitle {
    font-size: calc(0.75rem * var(--card-font-scale, 1));
    opacity: 0.75;
    text-align: center;
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
