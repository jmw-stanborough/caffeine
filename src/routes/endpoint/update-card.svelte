<script lang="ts">
  import { formatBytes } from "./metrics";
  import type { SystemUpdateItem } from "./types";

  interface Props {
    update: SystemUpdateItem;
    isBusy: boolean;
    isDownloading: boolean;
    isInstalling: boolean;
    onDownload: (id: string) => void;
    onInstall: (id: string) => void;
  }

  let {
    update,
    isBusy,
    isDownloading,
    isInstalling,
    onDownload,
    onInstall,
  }: Props = $props();
</script>

<div class="update-card" class:downloaded={update.is_downloaded}>
  <div class="update-main">
    <div class="update-badge-row">
      {#if update.kb_article_id}
        <span class="kb-badge">{update.kb_article_id}</span>
      {/if}
      {#if update.severity}
        <span class="severity-badge">{update.severity}</span>
      {/if}
      {#if update.size_bytes}
        <span class="size-badge">{formatBytes(update.size_bytes)}</span>
      {/if}
      {#if update.is_downloaded}
        <span class="status-pill ready">Downloaded</span>
      {:else}
        <span class="status-pill pending">Pending Download</span>
      {/if}
    </div>

    <div class="update-title" title={update.title}>{update.title}</div>

    {#if update.description}
      <div class="update-desc">{update.description}</div>
    {/if}

    {#if update.categories.length > 0}
      <div class="category-tags">
        {#each update.categories as cat}
          <span class="cat-pill">{cat}</span>
        {/each}
      </div>
    {/if}
  </div>

  <div class="update-card-actions">
    {#if !update.is_downloaded}
      <button
        class="btn-card-action"
        onclick={() => onDownload(update.id)}
        disabled={isBusy}
      >
        {isDownloading ? "Downloading..." : "Download"}
      </button>
    {:else}
      <button
        class="btn-card-action accent"
        onclick={() => onInstall(update.id)}
        disabled={isBusy}
      >
        {isInstalling ? "Installing..." : "Install"}
      </button>
    {/if}
  </div>
</div>

<style>
  .update-card {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem 0.85rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .update-card.downloaded {
    border-color: color-mix(in srgb, var(--color-primary) 40%, var(--color-border));
  }

  .update-main {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    flex: 1;
    min-width: 0;
  }

  .update-badge-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .kb-badge {
    font-size: 0.72rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
    background-color: var(--color-base);
    color: var(--color-text);
    border: solid var(--width-border) var(--color-border);
  }

  .severity-badge,
  .size-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.35rem;
    border-radius: 0.2rem;
    background-color: var(--color-base);
    color: var(--color-text-muted);
  }

  .status-pill {
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
  }

  .status-pill.ready {
    background-color: var(--color-primary);
    color: var(--color-base);
  }

  .status-pill.pending {
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .update-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text);
    line-height: 1.3;
  }

  .update-desc {
    font-size: 0.78rem;
    color: var(--color-text-muted);
    line-height: 1.35;
  }

  .category-tags {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .cat-pill {
    font-size: 0.68rem;
    color: var(--color-text-muted);
  }

  .btn-card-action {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.35rem 0.75rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .btn-card-action:hover:not(:disabled) {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover);
  }

  .btn-card-action.accent {
    background-color: var(--color-secondary);
    border-color: var(--color-secondary);
    color: var(--color-base);
  }

  .btn-card-action.accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }
</style>
