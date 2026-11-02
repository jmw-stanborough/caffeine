<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import { Circle } from "$lib/icons";
  import EventHistoryList, { type ConnectionLogItem } from "$lib/components/event-history-list.svelte";
  import { DEFAULT_LOG_LIMIT } from "$lib/constants";
  import { formatRelativeTime } from "$lib/format";
  import { invoke } from "@tauri-apps/api/core";
  import PrinterTagsRow from "./printer-tags-row.svelte";
  import PrinterTonerList from "./printer-toner-list.svelte";
  import { getEffectivePrinterStatus, parseStatusChips } from "./tags";
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
    canEdit = true,
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
    canEdit?: boolean;
    onwebui?: (printer: Printer) => void;
    onconfigure?: (printer: Printer) => void;
  } = $props();

  let relativeTime = $derived(formatRelativeTime(lastEventAt, now));
  let isConnected = $derived(status === "online" || status === "degraded");
  let connectionLabel = $derived(
    lastEvent === "connected" || (isConnected && lastEvent !== "disconnected")
      ? "Connected"
      : "Disconnected",
  );
  let displayModel = $derived(detectedModel || printer?.model);
  let statusChips = $derived(parseStatusChips(statusMessage));

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

  let activeTab = $state<"details" | "history">("details");
  let logs = $state<ConnectionLogItem[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      activeTab = "details";
      if (printer) {
        fetchLogs(printer.id);
      }
    }
  });

  async function fetchLogs(printerId: string) {
    logsLoading = true;
    logsError = null;
    try {
      logs = await invoke<ConnectionLogItem[]>("get_printer_logs", {
        printerId,
        limit: DEFAULT_LOG_LIMIT,
      });
    } catch (err) {
      logsError = String(err);
    } finally {
      logsLoading = false;
    }
  }

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
          <Circle
            size={10}
            class="status-dot {effectiveVisualStatus}"
            fill="var(--status-color, var(--color-border))"
            aria-hidden="true"
          />
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
        {#if onconfigure && canEdit}
          <button
            type="button"
            class="btn-action btn-config"
            onclick={handleConfigure}
            aria-label="Configure {printer.label}"
            title="Configure Printer"
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

    <div class="tab-nav">
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === "details"}
        onclick={() => (activeTab = "details")}
      >
        Details
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === "history"}
        onclick={() => (activeTab = "history")}
      >
        Event History
      </button>
    </div>

    {#if activeTab === "details"}

    {#if statusChips.length > 0 || printer.description}
      <div class="modal-description-group">
        {#if statusChips.length > 0}
          <div class="status-row">
            <span class="status-label">Status:</span>
            <div class="status-chips">
              {#each statusChips as chip (chip.id)}
                <span class="printer-tag {chip.className}">{chip.label}</span>
              {/each}
            </div>
          </div>
        {/if}
        {#if printer.description}
          <p class="card-description">{printer.description}</p>
        {/if}
      </div>
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
        {#if lastEventAt}
          <span class="card-since">
            {connectionLabel} {relativeTime}
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
  {:else}
    <EventHistoryList
      {logs}
      loading={logsLoading}
      error={logsError}
      onclose={() => (open = false)}
    />
  {/if}
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

  .card-title-row :global(.status-dot) {
    color: var(--status-color, var(--color-border));
    flex-shrink: 0;
  }

  .card-title-row :global(.status-dot.online) {
    color: var(--color-success);
  }

  .card-title-row :global(.status-dot.degraded) {
    color: var(--color-warning);
  }

  .card-title-row :global(.status-dot.offline) {
    color: var(--color-failure);
  }

  .card-title-row :global(.status-dot.checking) {
    color: var(--color-text-muted);
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

  .modal-description-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .status-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
    user-select: none;
  }

  .status-chips {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .status-chips .printer-tag {
    font-size: 0.72rem;
    padding: 0.18rem 0.45rem;
  }

  .status-chips .tag-status {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
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
