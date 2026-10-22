<script lang="ts">
  import { CARD_SCALE_STEP } from "$lib/constants";
  import type { Snippet } from "svelte";

  let {
    title,
    countText,
    isNocMode,
    errorMessage = null,
    onAdjustScale,
    onToggleNoc,
    onRetry = undefined,
    extraActions = undefined,
  }: {
    title: string;
    countText: string;
    isNocMode: boolean;
    errorMessage?: string | null;
    onAdjustScale: (delta: number) => void;
    onToggleNoc: () => void;
    onRetry?: () => void;
    extraActions?: Snippet;
  } = $props();
</script>

<header class="page-header">
  <div class="header-titles">
    <h1 class="dashboard-title">{title}</h1>
    <span class="item-count">{countText}</span>
  </div>
  <div class="header-actions">
    {#if extraActions}
      {@render extraActions()}
    {/if}

    <div class="zoom-controls">
      <button
        type="button"
        class="btn-zoom"
        onclick={() => onAdjustScale(-CARD_SCALE_STEP)}
        aria-label="Decrease card size"
      >
        −
      </button>
      <button
        type="button"
        class="btn-zoom"
        onclick={() => onAdjustScale(CARD_SCALE_STEP)}
        aria-label="Increase card size"
      >
        +
      </button>
    </div>

    <button
      type="button"
      class="btn-noc"
      onclick={onToggleNoc}
    >
      {isNocMode ? "Exit NOC (Esc)" : "NOC Mode"}
    </button>
  </div>
</header>

{#if errorMessage}
  <div class="error-banner">
    <span>{errorMessage}</span>
    {#if onRetry}
      <button type="button" class="btn-retry" onclick={onRetry}>Retry</button>
    {/if}
  </div>
{/if}

<style>
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

  .dashboard-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text);
    white-space: nowrap;
  }

  .item-count {
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
</style>
