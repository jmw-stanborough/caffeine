<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import { formatRelativeTime } from "$lib/format";
  import PrinterTagsRow from "./printer-tags-row.svelte";
  import PrinterTonerList from "./printer-toner-list.svelte";
  import { getEffectivePrinterStatus } from "./tags";
  import type { Printer, PrinterState, TonerSupply } from "./types";

  let {
    open = $bindable(false),
    printer = undefined,
    status = "checking",
    latencyMs = undefined,
    lastEvent = undefined,
    lastEventAt = undefined,
    toners = [],
    detectedMake = undefined,
    detectedModel = undefined,
    detectedLocation = undefined,
    serialNumber = undefined,
    pageCount = undefined,
    printerState = undefined,
    statusMessage = undefined,
    paperJam = false,
    outOfPaper = false,
    isColor = undefined,
    isDuplex = undefined,
    now = Date.now(),
    onwebui,
    onconfigure,
  }: {
    open: boolean;
    printer?: Printer;
    status?: PrinterState["status"];
    latencyMs?: number;
    lastEvent?: PrinterState["lastEvent"];
    lastEventAt?: string;
    toners?: TonerSupply[];
    detectedMake?: string;
    detectedModel?: string;
    detectedLocation?: string;
    serialNumber?: string;
    pageCount?: number;
    printerState?: string;
    statusMessage?: string;
    paperJam?: boolean;
    outOfPaper?: boolean;
    isColor?: boolean;
    isDuplex?: boolean;
    now?: number;
    onwebui?: (printer: Printer) => void;
    onconfigure?: (printer: Printer) => void;
  } = $props();

  let relativeTime = $derived(formatRelativeTime(lastEventAt, now));
  let displayModel = $derived(printer?.model || detectedModel);

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

  function handleWebUi() {
    if (!printer) return;
    const target = printer;
    open = false;
    onwebui?.(target);
  }

  function handleConfigure() {
    if (!printer) return;
    const target = printer;
    open = false;
    onconfigure?.(target);
  }
</script>

{#if printer}
  <Modal
    bind:open
    maxWidth="34rem"
    statusVariant={effectiveVisualStatus}
    ariaLabel="{printer.label} details"
  >
    <div class="modal-header">
      <div class="title-group">
        <div class="card-title-row">
          <span class="status-dot" aria-hidden="true"></span>
          <h2 class="modal-title" title={printer.label}>{printer.label}</h2>
        </div>

        {#if displayModel}
          <span class="model-name" title={displayModel}>{displayModel}</span>
        {/if}
      </div>

      <div class="header-actions">
        {#if onwebui}
          <button
            type="button"
            class="btn-action webui-btn"
            onclick={handleWebUi}
            title="Open internal web interface"
          >
            Web UI ↗
          </button>
        {/if}
        {#if onconfigure}
          <button
            type="button"
            class="btn-action btn-config"
            onclick={handleConfigure}
            aria-label="Configure {printer.label}"
            title="Configure Printer & Event History"
          >
            ⚙
          </button>
        {/if}
        <button
          type="button"
          class="btn-close"
          onclick={() => (open = false)}
          aria-label="Close details"
        >
          ✕
        </button>
      </div>
    </div>

    {#if printer.description || statusMessage}
      <p class="card-description">
        {statusMessage
          ? `${statusMessage}${printer.description ? ` — ${printer.description}` : ""}`
          : printer.description}
      </p>
    {/if}

    <PrinterTonerList
      {toners}
      {pageCount}
      {status}
      snmpCommunity={printer.snmp_community}
    />

    <div class="card-footer">
      <div class="footer-left">
        <PrinterTagsRow
          {printer}
          {toners}
          {detectedMake}
          {detectedModel}
          {detectedLocation}
          {printerState}
          {statusMessage}
          {paperJam}
          {outOfPaper}
          {isColor}
          {isDuplex}
          showMake={true}
          showLocation={true}
          showHiddenCount={false}
        />

        <div class="host-line">
          <span class="card-host selectable">{printer.host}</span>
          {#if serialNumber}
            <span class="serial-tag selectable">SN: {serialNumber}</span>
          {/if}
        </div>
        {#if lastEvent && lastEventAt}
          <span class="card-since">
            {lastEvent === "connected" ? "Connected" : "Disconnected"} {relativeTime}
          </span>
        {/if}
      </div>

      <span class="card-latency">
        {#if latencyMs !== undefined}
          {latencyMs} ms
        {:else if status === "checking"}
          ...
        {:else}
          —
        {/if}
      </span>
    </div>
  </Modal>
{/if}

<style>
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
  }

  .status-dot {
    width: 0.65rem;
    height: 0.65rem;
    border-radius: 50%;
    background-color: var(--status-color, var(--color-border));
    flex-shrink: 0;
  }

  .modal-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .model-name {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--color-text);
    opacity: 0.85;
    padding-left: 1.15rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .card-description {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text);
    opacity: 0.9;
  }

  .card-footer {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    font-size: 0.78rem;
    color: var(--color-text-muted);
  }

  .footer-left {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .host-line {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .card-host {
    font-family: monospace;
    font-weight: 600;
    color: var(--color-text);
  }

  .serial-tag {
    font-family: monospace;
    font-size: 0.72rem;
    opacity: 0.8;
  }

  .card-since {
    font-size: 0.72rem;
    color: var(--color-text-muted);
    opacity: 0.85;
  }

  .card-latency {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    flex-shrink: 0;
  }
</style>
