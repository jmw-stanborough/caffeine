<script lang="ts">
  import DashboardHeader from "$lib/components/dashboard-header.svelte";
  import {
    DEFAULT_CARD_SCALE,
    DND_FLIP_DURATION_MS,
    PRINTER_CARD_MIN_WIDTH_REM,
    PRINTER_NOC_MIN_WIDTH_REM,
  } from "$lib/constants";
  import { clampCardScale } from "$lib/format";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import PrinterCard from "./printer-card.svelte";
  import PrinterDetailModal from "./printer-detail-modal.svelte";
  import PrinterFilterModal from "./printer-filter-modal.svelte";
  import PrinterModal from "./printer-modal.svelte";
  import PrinterWebview from "./printer-webview.svelte";
  import { PrinterStore } from "./printers.svelte";
  import { filterPrintersByTags } from "./tags";
  import type { Printer } from "./types";

  const appWindow = getCurrentWindow();
  const store = new PrinterStore();

  let isModalOpen = $state(false);
  let editingPrinter = $state<Printer | undefined>(undefined);
  let isDetailModalOpen = $state(false);
  let detailPrinterId = $state<string | null>(null);
  let isFilterModalOpen = $state(false);
  let isNocMode = $state(false);
  let cardScale = $state(DEFAULT_CARD_SCALE);
  let activeWebPrinter = $state<Printer | null>(null);

  let activeFilterCount = $derived(
    store.filterAllTags.length + store.filterAnyTags.length,
  );
  let isFiltered = $derived(activeFilterCount > 0);
  let filteredPrinters = $derived(
    filterPrintersByTags(
      store.items,
      store.statusMap,
      store.filterAllTags,
      store.filterAnyTags,
    ),
  );

  let detailPrinter = $derived(
    detailPrinterId
      ? store.items.find((p) => p.id === detailPrinterId)
      : undefined,
  );
  let detailPrinterState = $derived(
    detailPrinterId ? store.statusMap[detailPrinterId] : undefined,
  );

  function adjustScale(delta: number) {
    cardScale = clampCardScale(cardScale, delta);
  }

  async function setNocMode(active: boolean) {
    if (active) {
      isDetailModalOpen = false;
      isModalOpen = false;
      isFilterModalOpen = false;
      activeWebPrinter = null;
    }
    isNocMode = active;
    try {
      await appWindow.setFullscreen(active);
    } catch (err) {
      console.error("Failed to set fullscreen:", err);
    }
  }

  function openWebInterface(printer: Printer) {
    if (isNocMode) return;
    isDetailModalOpen = false;
    activeWebPrinter = printer;
  }

  function openDetailModal(printer: Printer) {
    if (isNocMode) return;
    isFilterModalOpen = false;
    detailPrinterId = printer.id;
    isDetailModalOpen = true;
  }

  function openFilterModal() {
    if (isNocMode) return;
    isDetailModalOpen = false;
    isModalOpen = false;
    isFilterModalOpen = true;
  }

  function openEditModal(printer?: Printer) {
    if (isNocMode) return;
    isDetailModalOpen = false;
    isFilterModalOpen = false;
    activeWebPrinter = null;
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
        } else if (isFilterModalOpen) {
          isFilterModalOpen = false;
        } else if (activeWebPrinter) {
          activeWebPrinter = null;
        } else if (isNocMode) {
          setNocMode(false);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      store.destroy();
      window.removeEventListener("keydown", handleKeyDown);
      appWindow.setFullscreen(false).catch(() => {});
    };
  });
</script>

{#if activeWebPrinter}
  <PrinterWebview
    printer={activeWebPrinter}
    printerState={store.statusMap[activeWebPrinter.id]}
    onclose={() => (activeWebPrinter = null)}
    onconfigure={(p) => openEditModal(p)}
  />
{:else}
  <div
    class="page-container"
    class:noc-layout={isNocMode}
    style:--card-min-width="{PRINTER_CARD_MIN_WIDTH_REM * cardScale}rem"
    style:--noc-min-width="{PRINTER_NOC_MIN_WIDTH_REM * cardScale}rem"
    style:--card-font-scale={cardScale}
  >
    <DashboardHeader
      title={isNocMode ? "Printer NOC Monitor" : "Printers"}
      countText={isFiltered
        ? `${filteredPrinters.length} of ${store.items.length} shown`
        : `${store.items.length} configured`}
      {isNocMode}
      errorMessage={store.errorMessage}
      onAdjustScale={adjustScale}
      onToggleNoc={() => setNocMode(!isNocMode)}
      onRetry={store.fetchPrinters}
    >
      {#snippet extraActions()}
        {#if !isNocMode}
          <button
            type="button"
            class="btn-filter"
            class:active={isFiltered}
            onclick={openFilterModal}
            title="Configure tag filters"
          >
            <span>⚙ Filter</span>
            {#if isFiltered}
              <span class="filter-count-badge">{activeFilterCount}</span>
            {/if}
          </button>
        {/if}
      {/snippet}
    </DashboardHeader>

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
          flipDurationMs: DND_FLIP_DURATION_MS,
          dragDisabled: isNocMode || isFiltered,
          dropTargetStyle: {},
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
          <div animate:flip={{ duration: DND_FLIP_DURATION_MS }} class="drag-wrapper">
            <PrinterCard
              {printer}
              nocMode={isNocMode}
              status={printerState?.status ?? "checking"}
              toners={printerState?.toners ?? []}
              detectedMake={printerState?.detectedMake}
              detectedModel={printerState?.detectedModel}
              printerState={printerState?.printerState}
              statusMessage={printerState?.statusMessage}
              paperJam={printerState?.paperJam ?? false}
              outOfPaper={printerState?.outOfPaper ?? false}
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
  detectedMake={detailPrinterState?.detectedMake}
  detectedModel={detailPrinterState?.detectedModel}
  serialNumber={detailPrinterState?.serialNumber}
  pageCount={detailPrinterState?.pageCount}
  printerState={detailPrinterState?.printerState}
  statusMessage={detailPrinterState?.statusMessage}
  paperJam={detailPrinterState?.paperJam ?? false}
  outOfPaper={detailPrinterState?.outOfPaper ?? false}
  now={store.now}
  onwebui={(p) => openWebInterface(p)}
  onconfigure={(p) => openEditModal(p)}
/>

<PrinterFilterModal
  bind:open={isFilterModalOpen}
  {store}
/>

<PrinterModal
  bind:open={isModalOpen}
  printer={editingPrinter}
  onsave={(saved, id) => store.savePrinter(saved, id)}
  ondelete={(id) => store.deletePrinter(id)}
/>

<style>
  .btn-filter {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
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

  .btn-filter:hover {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .btn-filter.active {
    border-color: var(--color-primary);
    color: var(--color-text);
    background-color: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
  }

  .filter-count-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 0.85rem;
    height: 0.85rem;
    padding: 1px 0.2rem 0;
    border-radius: 999px;
    background-color: var(--color-primary);
    color: var(--color-base);
    font-size: 0.62rem;
    font-weight: 700;
    line-height: 1;
    box-sizing: border-box;
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
</style>
