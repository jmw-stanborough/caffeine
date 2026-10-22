<script lang="ts">
  import type { Printer, PrinterState, TonerSupply } from "./types";

  let {
    printer,
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
    nocMode = false,
    onclick,
    onconfigure,
  }: {
    printer: Printer;
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
    nocMode?: boolean;
    onclick?: () => void;
    onconfigure?: () => void;
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
  let displayModel = $derived(printer.model || detectedModel);

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
      return "OK";
    }
    if (toner.level >= 0) {
      return `${toner.level}`;
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

  function isLowToner(toner: TonerSupply): boolean {
    if (toner.supply_type === "waste_toner" || toner.supply_type === "waste_ink") {
      return false;
    }
    return toner.percent !== undefined && toner.percent !== null && toner.percent <= 15;
  }

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

  function handleConfigureKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.stopPropagation();
    }
  }
</script>

<div
  class="printer-card {status}"
  class:noc={nocMode}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={handleKeyDown}
  title={nocMode ? printer.label : `Click to open ${printer.label} web interface`}
>
  <div class="card-header">
    <div class="title-group">
      <div class="card-title-row">
        {#if !nocMode}
          <span class="status-dot" aria-hidden="true"></span>
        {/if}
        <h2 class="card-label">{printer.label}</h2>
        {#if printerState && !nocMode}
          <span class="badge-state">{printerState}</span>
        {/if}
      </div>

      {#if !nocMode}
        <div class="meta-row">
          {#if displayModel}
            <span class="meta-item model-name">{displayModel}</span>
          {/if}
          {#if printer.location}
            <span class="meta-item location-badge">{printer.location}</span>
          {/if}
        </div>
      {/if}
    </div>

    {#if !nocMode}
      <div class="card-actions">
        <span class="webui-hint" aria-hidden="true">Web UI ↗</span>
        <button
          type="button"
          class="btn-config"
          onclick={handleConfigureClick}
          onkeydown={handleConfigureKeyDown}
          aria-label="Configure {printer.label}"
          title="Configure Printer & Event History"
        >
          ⚙
        </button>
      </div>
    {/if}
  </div>

  {#if !nocMode && (printer.description || statusMessage)}
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
          <span>No SNMP / IPP supply data reported by host ({printer.snmp_community || "public"})</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="card-footer">
    <div class="footer-left">
      <div class="host-line">
        <span class="card-host">{printer.host}</span>
        {#if serialNumber && !nocMode}
          <span class="serial-tag">SN: {serialNumber}</span>
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

<style>
  .printer-card {
    border: solid var(--width-border) var(--status-color, var(--color-border));
    border-radius: 0.5rem;
    padding: 1.15rem 1.25rem;
    min-height: var(--card-min-height, 15.5rem);
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 0.85rem;
    cursor: pointer;
    user-select: none;
    transition: border-color 0.15s ease, background-color 0.15s ease, transform 0.12s ease;
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
  }

  .title-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
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
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 50%;
    background-color: var(--status-color, var(--color-border));
    flex-shrink: 0;
  }

  .card-label {
    margin: 0;
    font-size: calc(1.1rem * var(--card-font-scale, 1));
    font-weight: 700;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge-state {
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    font-weight: 600;
    padding: 0.12rem 0.4rem;
    border-radius: 0.25rem;
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    font-size: calc(0.78rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
  }

  .model-name {
    font-weight: 500;
    color: var(--color-text);
    opacity: 0.9;
  }

  .location-badge {
    padding: 0.05rem 0.35rem;
    border-radius: 0.25rem;
    background-color: color-mix(in srgb, var(--color-base) 50%, transparent);
    border: solid var(--width-border) var(--color-border);
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .webui-hint {
    font-size: calc(0.72rem * var(--card-font-scale, 1));
    font-weight: 600;
    color: var(--color-link);
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    background-color: color-mix(in srgb, var(--color-base) 60%, transparent);
    border: solid var(--width-border) var(--color-border);
  }

  .btn-config {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    border-radius: 0.35rem;
    width: 1.85rem;
    height: 1.85rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.95rem;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
  }

  .btn-config:hover {
    background-color: var(--color-base);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .card-description {
    margin: 0;
    font-size: calc(0.8rem * var(--card-font-scale, 1));
    color: var(--color-text);
    opacity: 0.9;
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .toners-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    border-radius: 0.4rem;
    background-color: color-mix(in srgb, var(--color-base) 68%, transparent);
    border: solid var(--width-border) color-mix(in srgb, var(--color-border) 80%, transparent);
    margin-block: auto;
  }

  .toners-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: calc(0.72rem * var(--card-font-scale, 1));
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
    gap: 0.22rem;
  }

  .toner-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: calc(0.78rem * var(--card-font-scale, 1));
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
    padding: 0.65rem 0.25rem;
    text-align: center;
    font-size: calc(0.78rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    font-style: italic;
  }

  .card-footer {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    font-size: calc(0.75rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    margin-top: auto;
  }

  .footer-left {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .host-line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .card-host {
    font-family: monospace;
    font-weight: 600;
    color: var(--color-text);
  }

  .serial-tag {
    font-family: monospace;
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    opacity: 0.8;
  }

  .card-since {
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    opacity: 0.85;
  }

  .card-latency {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }
</style>
