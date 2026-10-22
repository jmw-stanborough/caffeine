<script lang="ts">
  import { DEFAULT_SNMP_COMMUNITY } from "$lib/constants";
  import { isEmptyToner, isLowToner } from "./tags";
  import type { StatusKind, TonerSupply } from "./types";

  const SNMP_LEVEL_SOME_REMAINING = -3;
  const SOME_REMAINING_BAR_WIDTH_PCT = 60;
  const MIN_VISIBLE_BAR_WIDTH_PCT = 2;
  const MAX_BAR_WIDTH_PCT = 100;

  const TONER_COLOR_MAP: Record<string, { hex: string; shortLabel: string }> = {
    cyan: { hex: "#00b4d8", shortLabel: "C" },
    magenta: { hex: "#e63980", shortLabel: "M" },
    yellow: { hex: "#eab308", shortLabel: "Y" },
    black: { hex: "#27272a", shortLabel: "K" },
    waste: { hex: "#78716c", shortLabel: "W" },
    drum: { hex: "#8b5cf6", shortLabel: "D" },
    maintenance: { hex: "#64748b", shortLabel: "S" },
  };

  let {
    toners = [],
    pageCount = undefined,
    status = "checking",
    snmpCommunity = DEFAULT_SNMP_COMMUNITY,
  }: {
    toners?: TonerSupply[];
    pageCount?: number;
    status?: StatusKind;
    snmpCommunity?: string;
  } = $props();

  function getTonerBarColor(toner: TonerSupply): string {
    return (
      TONER_COLOR_MAP[toner.color.toLowerCase()]?.hex ?? "var(--color-primary)"
    );
  }

  function getTonerShortLabel(toner: TonerSupply): string {
    return TONER_COLOR_MAP[toner.color.toLowerCase()]?.shortLabel ?? "S";
  }

  function formatLevelText(toner: TonerSupply): string {
    if (toner.percent !== undefined && toner.percent !== null) {
      return `${toner.percent}%`;
    }
    if (toner.level === SNMP_LEVEL_SOME_REMAINING) {
      return "Available";
    }
    if (toner.level >= 0) {
      return `${toner.level}%`;
    }
    return "N/A";
  }

  function getBarWidth(toner: TonerSupply): number {
    if (toner.percent !== undefined && toner.percent !== null) {
      if (toner.percent <= 0) return 0;
      return Math.max(
        MIN_VISIBLE_BAR_WIDTH_PCT,
        Math.min(MAX_BAR_WIDTH_PCT, toner.percent),
      );
    }
    if (toner.level === SNMP_LEVEL_SOME_REMAINING) {
      return SOME_REMAINING_BAR_WIDTH_PCT;
    }
    return 0;
  }
</script>

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
        {@const empty = isEmptyToner(toner)}
        {@const low = isLowToner(toner)}
        <div class="toner-row" class:low-supply={low || empty}>
          <div class="toner-info">
            <span
              class="toner-swatch"
              style:background-color={barColor}
              aria-hidden="true"
            >
              {getTonerShortLabel(toner)}
            </span>
            <span class="toner-name" title={toner.name}>{toner.name}</span>
            {#if empty}
              <span class="empty-badge">EMPTY</span>
            {:else if low}
              <span class="low-badge">LOW</span>
            {/if}
            <span class="toner-pct">{formatLevelText(toner)}</span>
          </div>
          <div
            class="toner-track"
            role="progressbar"
            aria-valuenow={toner.percent ?? 0}
            aria-valuemin={0}
            aria-valuemax={MAX_BAR_WIDTH_PCT}
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
        <span>No toner data reported by host ({snmpCommunity || DEFAULT_SNMP_COMMUNITY})</span>
      {/if}
    </div>
  {/if}
</div>

<style>
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

  .empty-badge,
  .low-badge {
    font-size: 0.65rem;
    font-weight: 800;
    padding: 0.05rem 0.3rem;
    border-radius: 0.2rem;
    color: #1c1917;
  }

  .empty-badge {
    background-color: var(--color-failure);
  }

  .low-badge {
    background-color: var(--color-warning);
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
</style>
