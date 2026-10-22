<script lang="ts">
  import { activateOnEnterOrSpace } from "$lib/format";
  import PrinterTagsRow from "./printer-tags-row.svelte";
  import { getEffectivePrinterStatus, isLowToner } from "./tags";
  import type { Printer, PrinterState, TonerSupply } from "./types";

  let {
    printer,
    status = "checking",
    toners = [],
    detectedMake = undefined,
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
    detectedMake?: string;
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
  let hasLowToner = $derived(toners.some(isLowToner));

  let effectiveVisualStatus = $derived(
    getEffectivePrinterStatus({
      status,
      toners,
      paperJam,
      outOfPaper,
      printerState,
      statusMessage,
    }),
  );

  function handleConfigureClick(e: MouseEvent) {
    e.stopPropagation();
    onconfigure?.();
  }

  function handleActionKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      // Prevent Enter/Space on nested action buttons from triggering the parent card click.
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
  onkeydown={(e) => activateOnEnterOrSpace(e, onclick)}
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

  <PrinterTagsRow
    {printer}
    {toners}
    {detectedMake}
    {detectedModel}
    {printerState}
    {statusMessage}
    {paperJam}
    {outOfPaper}
  />
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

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-shrink: 0;
  }
</style>
