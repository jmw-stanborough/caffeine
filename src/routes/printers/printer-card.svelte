<script lang="ts">
  import {
    getEffectivePrinterStatus,
    hasOutOfPaper,
    hasPaperJam,
    inferPrinterMake,
    isEmptyToner,
    isLowToner,
  } from "./tags";
  import type { Printer, PrinterState, TonerSupply } from "./types";

  let {
    printer,
    status = "checking",
    toners = [],
    detectedModel = undefined,
    printerState = undefined,
    statusMessage = undefined,
    paperJam = false,
    outOfPaper = false,
    nocMode = false,
    onclick,
    onwebui,
    onconfigure,
  }: {
    printer: Printer;
    status?: PrinterState["status"];
    toners?: TonerSupply[];
    detectedModel?: string;
    printerState?: string;
    statusMessage?: string;
    paperJam?: boolean;
    outOfPaper?: boolean;
    nocMode?: boolean;
    onclick?: () => void;
    onwebui?: () => void;
    onconfigure?: () => void;
  } = $props();

  let displayModel = $derived(printer.model || detectedModel);
  let makeTag = $derived(inferPrinterMake(printer, detectedModel));
  let isJammed = $derived(hasPaperJam({ paperJam, printerState, statusMessage }));
  let isNoPaper = $derived(hasOutOfPaper({ outOfPaper, printerState, statusMessage }));
  let hasEmptyToner = $derived(toners.some((t) => isEmptyToner(t)));
  let hasLowToner = $derived(toners.some((t) => isLowToner(t)));

  let effectiveVisualStatus = $derived(
    getEffectivePrinterStatus({
      status,
      toners,
      paperJam,
      outOfPaper,
      printerState,
      statusMessage,
    })
  );

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.key === "Enter" || e.key === " ") && onclick) {
      e.preventDefault();
      onclick();
    }
  }

  function handleConfigureClick(e: MouseEvent) {
    e.stopPropagation();
    onconfigure?.();
  }

  function handleActionKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.stopPropagation();
    }
  }

  function handleWebUiClick(e: MouseEvent) {
    e.stopPropagation();
    onwebui?.();
  }
</script>

<div
  class="printer-card {effectiveVisualStatus}"
  class:noc={nocMode}
  class:has-low-toner={hasLowToner}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={handleKeyDown}
  title={nocMode ? printer.label : `Click to view ${printer.label} details`}
>
  <div class="card-header">
    <div class="title-group">
      <div class="card-title-row">
        {#if !nocMode}
          <span class="status-dot" aria-hidden="true"></span>
        {/if}
        <h2 class="card-label" title={printer.label}>{printer.label}</h2>
      </div>

      {#if !nocMode && displayModel}
        <span class="model-name" title={displayModel}>{displayModel}</span>
      {/if}
    </div>

    {#if !nocMode}
      <div class="card-actions">
        <button
          type="button"
          class="btn-action webui-btn"
          onclick={handleWebUiClick}
          onkeydown={handleActionKeyDown}
          title="Open internal web interface"
        >
          Web UI ↗
        </button>
        <button
          type="button"
          class="btn-action btn-config"
          onclick={handleConfigureClick}
          onkeydown={handleActionKeyDown}
          aria-label="Configure {printer.label}"
          title="Configure Printer & Event History"
        >
          ⚙
        </button>
      </div>
    {/if}
  </div>

  {#if makeTag || printer.location || printerState || isJammed || isNoPaper || hasEmptyToner || hasLowToner}
    <div class="tags-row">
      {#if makeTag}
        <span class="printer-tag tag-make" title="Make: {makeTag}">{makeTag}</span>
      {/if}
      {#if printer.location}
        <span class="printer-tag tag-location" title={printer.location}>{printer.location}</span>
      {/if}
      {#if printerState}
        <span class="printer-tag tag-state">{printerState}</span>
      {/if}
      {#if isJammed}
        <span class="printer-tag tag-failure" title="Printer has a paper jam">Paper Jam</span>
      {/if}
      {#if isNoPaper}
        <span class="printer-tag tag-failure" title="Printer is out of paper">Out of Paper</span>
      {/if}
      {#if hasEmptyToner}
        <span class="printer-tag tag-failure" title="One or more cartridges have reached 0% toner">Toner Empty</span>
      {/if}
      {#if hasLowToner}
        <span class="printer-tag tag-warning" title="One or more cartridges are low on toner">Toner Low</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .printer-card {
    border: solid var(--width-border) var(--status-color, var(--color-border));
    border-radius: 0.5rem;
    padding: 0.85rem 1.1rem;
    width: 100%;
    height: 100%;
    min-height: 5rem;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 0.5rem;
    cursor: pointer;
    user-select: none;
    transition: border-color 0.15s ease, background-color 0.15s ease, box-shadow 0.15s ease;
  }

  .printer-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .printer-card.noc {
    cursor: default;
  }

  .printer-card.online {
    --status-color: var(--color-success);
  }

  .printer-card.degraded {
    --status-color: var(--color-warning);
  }

  .printer-card.offline {
    --status-color: var(--color-failure);
  }

  .printer-card:is(.online, .degraded, .offline) {
    background-color: color-mix(in srgb, var(--status-color) 26%, var(--color-surface));
  }

  .printer-card:is(.online, .degraded, .offline):hover {
    background-color: color-mix(in srgb, var(--status-color) 36%, var(--color-surface));
  }

  .printer-card.checking {
    --status-color: var(--color-text-muted);
    background-color: color-mix(in srgb, var(--color-surface) 65%, transparent);
    border-color: var(--color-border);
  }

  .printer-card.checking:hover {
    background-color: color-mix(in srgb, var(--color-surface) 88%, transparent);
    border-color: var(--color-text-muted);
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    min-width: 0;
    width: 100%;
  }

  .title-group {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
    flex: 1;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    flex: 1;
  }

  .status-dot {
    width: 0.65rem;
    height: 0.65rem;
    border-radius: 50%;
    background-color: var(--status-color, var(--color-border));
    flex-shrink: 0;
  }

  .card-label {
    margin: 0;
    font-size: calc(1.05rem * var(--card-font-scale, 1));
    font-weight: 700;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex: 1;
  }

  .model-name {
    font-size: calc(0.78rem * var(--card-font-scale, 1));
    font-weight: 500;
    color: var(--color-text);
    opacity: 0.85;
    padding-left: 1.15rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tags-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.35rem;
    min-width: 0;
  }

  .printer-tag {
    display: inline-flex;
    align-items: center;
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    font-weight: 600;
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    line-height: 1;
    border: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
    white-space: nowrap;
    box-sizing: border-box;
  }

  .tag-failure {
    background-color: var(--color-failure);
    color: #1c1917;
    border-color: color-mix(in srgb, var(--color-failure) 75%, #000000);
  }

  .tag-warning {
    background-color: var(--color-warning);
    color: #1c1917;
    border-color: color-mix(in srgb, var(--color-warning) 75%, #000000);
  }

  .tag-make,
  .tag-state,
  .tag-location {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  .btn-action {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    border-radius: 0.35rem;
    height: 1.85rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    white-space: nowrap;
    transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
  }

  .webui-btn {
    font-size: calc(0.72rem * var(--card-font-scale, 1));
    font-weight: 600;
    color: var(--color-link);
    padding-inline: 0.5rem;
  }

  .webui-btn:hover {
    background-color: var(--color-base);
    border-color: var(--color-link);
    color: var(--color-link-hover);
  }

  .btn-config {
    width: 1.85rem;
    font-size: 0.95rem;
  }

  .btn-config:hover {
    background-color: var(--color-base);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }
</style>
