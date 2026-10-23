<script lang="ts">
  import DashboardHeader from "$lib/components/dashboard-header.svelte";
  import SortModal from "$lib/components/sort-modal.svelte";
  import { Filter, Sort } from "$lib/icons";
  import {
    DND_FLIP_DURATION_MS,
    PRINTER_CARD_MIN_WIDTH_REM,
    PRINTER_NOC_MIN_WIDTH_REM,
  } from "$lib/constants";
  import { createNocController } from "$lib/noc.svelte";
  import { loadSavedSortRules, saveSortRules, sortEntities } from "$lib/sort/sort";
  import type { SortRule } from "$lib/sort/types";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import PrinterCard from "./printer-card.svelte";
  import PrinterDetailModal from "./printer-detail-modal.svelte";
  import PrinterFilterModal from "./printer-filter-modal.svelte";
  import PrinterModal from "./printer-modal.svelte";
  import PrinterWebview from "./printer-webview.svelte";
  import { printerStore } from "./printers.svelte";
  import { PRINTER_SORT_FIELDS, PRINTER_SORT_STORAGE_KEY } from "./sort";
  import { filterPrintersByTags } from "./tags";
  import type { Printer } from "./types";

  const store = printerStore;

  let isModalOpen = $state(false);
  let editingPrinter = $state<Printer | undefined>(undefined);
  let isDetailModalOpen = $state(false);
  let detailPrinterId = $state<string | null>(null);
  let isFilterModalOpen = $state(false);
  let isSortModalOpen = $state(false);
  let sortRules = $state<SortRule[]>(loadSavedSortRules(PRINTER_SORT_STORAGE_KEY));
  let activeWebPrinter = $state<Printer | null>(null);

  $effect(() => {
    saveSortRules(PRINTER_SORT_STORAGE_KEY, sortRules);
  });

  const noc = createNocController(() => {
    isDetailModalOpen = false;
    isModalOpen = false;
    isFilterModalOpen = false;
    isSortModalOpen = false;
    activeWebPrinter = null;
  });

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
  let isSorted = $derived(sortRules.length > 0);
  let displayPrinters = $derived(
    isSorted
      ? sortEntities(
          filteredPrinters,
          sortRules,
          PRINTER_SORT_FIELDS,
          store.statusMap,
        )
      : filteredPrinters,
  );

  let detailPrinter = $derived(
    detailPrinterId
      ? store.items.find((p) => p.id === detailPrinterId)
      : undefined,
  );
  let detailPrinterState = $derived(
    detailPrinterId ? store.statusMap[detailPrinterId] : undefined,
  );

  function openWebInterface(printer: Printer) {
    if (noc.isNocMode) return;
    isDetailModalOpen = false;
    activeWebPrinter = printer;
  }

  function openDetailModal(printer: Printer) {
    if (noc.isNocMode) return;
    isFilterModalOpen = false;
    isSortModalOpen = false;
    detailPrinterId = printer.id;
    isDetailModalOpen = true;
  }

  function openFilterModal() {
    if (noc.isNocMode) return;
    isDetailModalOpen = false;
    isModalOpen = false;
    isSortModalOpen = false;
    isFilterModalOpen = true;
  }

  function openSortModal() {
    if (noc.isNocMode) return;
    isDetailModalOpen = false;
    isModalOpen = false;
    isFilterModalOpen = false;
    isSortModalOpen = true;
  }

  function openEditModal(printer?: Printer) {
    if (noc.isNocMode) return;
    isDetailModalOpen = false;
    isFilterModalOpen = false;
    isSortModalOpen = false;
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
        } else if (isSortModalOpen) {
          isSortModalOpen = false;
        } else if (activeWebPrinter) {
          activeWebPrinter = null;
        } else if (noc.isNocMode) {
          noc.setNocMode(false).catch((err) => {
            console.error("Failed to exit NOC mode:", err);
          });
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      noc.cleanup();
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
    class:noc-layout={noc.isNocMode}
    style:--card-min-width="{PRINTER_CARD_MIN_WIDTH_REM * noc.cardScale}rem"
    style:--noc-min-width="{PRINTER_NOC_MIN_WIDTH_REM * noc.cardScale}rem"
    style:--card-font-scale={noc.cardScale}
  >
    <DashboardHeader
      title={noc.isNocMode ? "Printer NOC Monitor" : "Printers"}
      countText={isFiltered
        ? `${displayPrinters.length} of ${store.items.length} shown`
        : `${store.items.length} configured`}
      isNocMode={noc.isNocMode}
      errorMessage={store.errorMessage}
      onAdjustScale={noc.adjustScale}
      onToggleNoc={() => noc.setNocMode(!noc.isNocMode)}
      onRetry={store.fetchPrinters}
    >
      {#snippet extraActions()}
        {#if !noc.isNocMode}
          <button
            type="button"
            class="btn-sort"
            class:active={isSorted}
            onclick={openSortModal}
            title={isSorted ? `Sorting by ${sortRules.length} rule(s)` : "Configure sort rules"}
          >
            <Sort size={14} />
            <span>Sort</span>
            {#if isSorted}
              <span class="count-badge sort-count-badge">{sortRules.length}</span>
            {/if}
          </button>

          <button
            type="button"
            class="btn-filter"
            class:active={isFiltered}
            onclick={openFilterModal}
            title="Configure tag filters"
          >
            <Filter size={14} />
            <span>Filter</span>
            {#if isFiltered}
              <span class="count-badge filter-count-badge">{activeFilterCount}</span>
            {/if}
          </button>
        {/if}
      {/snippet}
    </DashboardHeader>

    {#if store.loading}
      <p class="status-message">Reading printer configuration...</p>
    {:else}
      {#if isFiltered && store.items.length > 0 && displayPrinters.length === 0}
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
        class:noc-grid={noc.isNocMode}
        use:dndzone={{
          items: displayPrinters,
          flipDurationMs: DND_FLIP_DURATION_MS,
          dragDisabled: noc.isNocMode || isFiltered || isSorted,
          dropTargetStyle: {},
        }}
        onconsider={(e) => {
          if (!isFiltered && !isSorted) {
            store.items = e.detail.items;
          }
        }}
        onfinalize={(e) => {
          if (!isFiltered && !isSorted) {
            store.reorderPrinters(e.detail.items);
          }
        }}
      >
        {#each displayPrinters as printer (printer.id)}
          {@const printerState = store.statusMap[printer.id]}
          <div animate:flip={{ duration: DND_FLIP_DURATION_MS }} class="drag-wrapper">
            <PrinterCard
              {printer}
              nocMode={noc.isNocMode}
              status={printerState?.status ?? "checking"}
              toners={printerState?.toners ?? []}
              detectedMake={printerState?.detectedMake}
              detectedModel={printerState?.detectedModel}
              detectedLocation={printerState?.detectedLocation}
              printerState={printerState?.printerState}
              statusMessage={printerState?.statusMessage}
              paperJam={printerState?.paperJam ?? false}
              outOfPaper={printerState?.outOfPaper ?? false}
              isColor={printerState?.isColor}
              isDuplex={printerState?.isDuplex}
              onclick={() => openDetailModal(printer)}
              onwebui={() => openWebInterface(printer)}
              onconfigure={() => openEditModal(printer)}
            />
          </div>
        {/each}

        {#if !noc.isNocMode}
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
  detectedLocation={detailPrinterState?.detectedLocation}
  serialNumber={detailPrinterState?.serialNumber}
  pageCount={detailPrinterState?.pageCount}
  printerState={detailPrinterState?.printerState}
  statusMessage={detailPrinterState?.statusMessage}
  paperJam={detailPrinterState?.paperJam ?? false}
  outOfPaper={detailPrinterState?.outOfPaper ?? false}
  isColor={detailPrinterState?.isColor}
  isDuplex={detailPrinterState?.isDuplex}
  now={store.now}
  onwebui={(p) => openWebInterface(p)}
  onconfigure={(p) => openEditModal(p)}
/>

<SortModal
  bind:open={isSortModalOpen}
  title="Sort Printers"
  bind:rules={sortRules}
  fields={PRINTER_SORT_FIELDS}
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
