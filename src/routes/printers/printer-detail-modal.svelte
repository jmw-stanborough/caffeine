<script lang="ts">
  import { inferPrinterMake, isLowToner } from "./tags";
  import type { Printer, PrinterState, TonerSupply } from "./types";

  let {
    open = $bindable(false),
    printer = undefined,
    status = "checking",
    latencyMs = undefined,
    lastEvent = undefined,
    lastEventAt = undefined,
    toners = [],
    detectedModel = undefined,
    serialNumber = undefined,
    pageCount = undefined,
    printerState = undefined,
    statusMessage = undefined,
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
    detectedModel?: string;
    serialNumber?: string;
    pageCount?: number;
    printerState?: string;
    statusMessage?: string;
    now?: number;
    onwebui?: (printer: Printer) => void;
    onconfigure?: (printer: Printer) => void;
  } = $props();

  function formatRelativeTime(dateString?: string, currentTime: number = Date.now()): string {
    if (!dateString) return "";
    const parsed = new Date(dateString);
    if (isNaN(parsed.getTime())) return "";

    const diffSec = Math.floor((currentTime - parsed.getTime()) / 1000);
    if (diffSec < 60) return "just now";
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
    return `${Math.floor(diffSec / 86400)}d ago`;
  }

  let relativeTime = $derived(formatRelativeTime(lastEventAt, now));
  let displayModel = $derived(printer?.model || detectedModel);
  let makeTag = $derived(inferPrinterMake(printer, detectedModel));
  let hasLowToner = $derived(toners.some((t) => isLowToner(t)));

  let effectiveVisualStatus = $derived(
    status === "offline"
      ? "offline"
      : hasLowToner
        ? "degraded"
        : status
  );

  function getTonerBarColor(toner: TonerSupply): string {
    switch (toner.color.toLowerCase()) {
      case "cyan":
        return "#00b4d8";
      case "magenta":
        return "#e63980";
      case "yellow":
        return "#eab308";
      case "black":
        return "#27272a";
      case "waste":
        return "#78716c";
      case "drum":
        return "#8b5cf6";
      case "maintenance":
        return "#64748b";
      default:
        return "var(--color-primary)";
    }
  }

  function getTonerShortLabel(toner: TonerSupply): string {
    const c = toner.color.toLowerCase();
    if (c === "cyan") return "C";
    if (c === "magenta") return "M";
    if (c === "yellow") return "Y";
    if (c === "black") return "K";
    if (c === "waste") return "W";
    if (c === "drum") return "D";
    return "S";
  }

  function formatLevelText(toner: TonerSupply): string {
    if (toner.percent !== undefined && toner.percent !== null) {
      return `${toner.percent}%`;
    }
    if (toner.level === -3) {
      return "Available";
    }
    if (toner.level >= 0) {
      return `${toner.level}%`;
    }
    return "N/A";
  }

  function getBarWidth(toner: TonerSupply): number {
    if (toner.percent !== undefined && toner.percent !== null) {
      return Math.max(2, Math.min(100, toner.percent));
    }
    if (toner.level === -3) {
      return 60;
    }
    return 0;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleBackdropKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
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

{#if open && printer}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeyDown}
    role="presentation"
  >
    <div
      class="modal-card {effectiveVisualStatus}"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-label="{printer.label} details"
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
          {statusMessage ? `${statusMessage}${printer.description ? ` — ${printer.description}` : ""}` : printer.description}
        </p>
      {/if}

      <div class="toners-section">
        <div class="toners-header">
          <span class="section-title">Toner & Supplies</span>
          {#if pageCount !== undefined && pageCount !== null}
            <span class="page-counter">{pageCount.toLocaleString()} pages</span>
          {/if}
        </div>

        {#if toners.length > 0}
          <div class="toners-list">
            {#each toners as toner (toner.name + toner.color)}
              {@const barColor = getTonerBarColor(toner)}
              {@const low = isLowToner(toner)}
              <div class="toner-row" class:low-supply={low}>
                <div class="toner-info">
                  <span
                    class="toner-swatch"
                    style:background-color={barColor}
                    aria-hidden="true"
                  >
                    {getTonerShortLabel(toner)}
                  </span>
                  <span class="toner-name" title={toner.name}>{toner.name}</span>
                  {#if low}
                    <span class="low-badge">LOW</span>
                  {/if}
                  <span class="toner-pct">{formatLevelText(toner)}</span>
                </div>
                <div
                  class="toner-track"
                  role="progressbar"
                  aria-valuenow={toner.percent ?? 0}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-label="{toner.name} level"
                >
                  <div
                    class="toner-fill"
                    style:width="{getBarWidth(toner)}%"
                    style:background-color={barColor}
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="toners-empty">
            {#if status === "checking"}
              <span>Polling printer supplies via SNMP / IPP...</span>
            {:else if status === "offline"}
              <span>Printer unreachable — toner levels unavailable</span>
            {:else}
              <span>No toner data reported by host ({printer.snmp_community || "public"})</span>
            {/if}
          </div>
        {/if}
      </div>

      <div class="card-footer">
        <div class="footer-left">
          {#if makeTag || printer.location || printerState || hasLowToner}
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
              {#if hasLowToner}
                <span class="printer-tag tag-warning" title="One or more cartridges are low on toner">Toner Low</span>
              {/if}
            </div>
          {/if}
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
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    box-sizing: border-box;
    z-index: 100;
  }

  .modal-card {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    width: 100%;
    max-width: 34rem;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    outline: none;
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
  }

  .modal-card.online {
    --status-color: var(--color-success);
  }

  .modal-card.degraded {
    --status-color: var(--color-warning);
  }

  .modal-card.offline {
    --status-color: var(--color-failure);
  }

  .modal-card.checking {
    --status-color: var(--color-text-muted);
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    min-width: 0;
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
  }

  .status-dot {
    width: 0.65rem;
    height: 0.65rem;
    border-radius: 50%;
    background-color: var(--status-color, var(--color-border));
    flex-shrink: 0;
  }

  .modal-title {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--color-text);
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
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-link);
    padding-inline: 0.55rem;
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

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
    padding: 0.25rem;
    margin-left: 0.15rem;
    transition: color 0.15s ease;
  }

  .btn-close:hover {
    color: var(--color-text);
  }

  .card-description {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text);
    opacity: 0.9;
  }

  .toners-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.75rem 0.85rem;
    border-radius: 0.4rem;
    background-color: color-mix(in srgb, var(--color-base) 68%, transparent);
    border: solid var(--width-border) color-mix(in srgb, var(--color-border) 80%, transparent);
  }

  .toners-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }

  .page-counter {
    font-variant-numeric: tabular-nums;
    text-transform: none;
    font-weight: 600;
  }

  .toners-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .toner-row {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .toner-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
  }

  .toner-swatch {
    width: 1.05rem;
    height: 1.05rem;
    border-radius: 0.2rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.65rem;
    font-weight: 800;
    color: #ffffff;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.65);
    flex-shrink: 0;
  }

  .toner-name {
    color: var(--color-text);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .low-badge {
    font-size: 0.65rem;
    font-weight: 800;
    padding: 0.05rem 0.3rem;
    border-radius: 0.2rem;
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .toner-pct {
    font-family: monospace;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
  }

  .toner-track {
    width: 100%;
    height: 0.55rem;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--color-surface-active) 75%, transparent);
    overflow: hidden;
    border: solid 1px color-mix(in srgb, var(--color-border) 70%, transparent);
  }

  .toner-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.35s ease;
  }

  .toners-empty {
    padding: 0.5rem 0.25rem;
    text-align: center;
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-style: italic;
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
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    line-height: 1;
    border: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
    white-space: nowrap;
    box-sizing: border-box;
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
