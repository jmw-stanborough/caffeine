<script lang="ts">
  import { formatBytes } from "./metrics";
  import type {
    SystemUpdateItem,
    UpdateOperationType,
    UpdateProgressEvent,
  } from "./types";
  import UpdateCardActions from "./updates/update-card-actions.svelte";
  import UpdateCardDetails from "./updates/update-card-details.svelte";

  interface Props {
    update: SystemUpdateItem;
    isSelected: boolean;
    isDownloading: boolean;
    isInstalling: boolean;
    progress?: UpdateProgressEvent | null;
    isQueued: boolean;
    queuedType?: UpdateOperationType;
    isBusy: boolean;
    rebootRequired: boolean;
    onToggleSelect: (id: string) => void;
    onDownload: (id: string) => void;
    onInstall: (id: string) => void;
    onDownloadAndInstall: (id: string) => void;
    onCancelQueue: (id: string) => void;
  }

  let {
    update,
    isSelected,
    isDownloading,
    isInstalling,
    progress,
    isQueued,
    queuedType,
    rebootRequired,
    onToggleSelect,
    onDownload,
    onInstall,
    onDownloadAndInstall,
    onCancelQueue,
  }: Props = $props();

  let isExpanded = $state(false);

  function getSeverityClass(severity?: string): string {
    if (!severity) return "sev-none";
    const s = severity.toLowerCase();
    if (s.includes("critical")) return "sev-critical";
    if (s.includes("important")) return "sev-important";
    if (s.includes("moderate")) return "sev-moderate";
    if (s.includes("low")) return "sev-low";
    return "sev-other";
  }
</script>

<div
  class="update-card"
  class:selected={isSelected}
  class:downloaded={update.is_downloaded}
  class:in-progress={isDownloading || isInstalling}
  class:queued={isQueued}
>
  <div class="card-left-col">
    <label class="checkbox-container" title="Select for batch action">
      <input
        type="checkbox"
        checked={isSelected}
        onchange={() => onToggleSelect(update.id)}
        class="card-checkbox"
      />
      <span class="custom-checkbox"></span>
    </label>
  </div>

  <div class="update-main">
    <div class="update-badge-row">
      {#if update.kb_article_id}
        <span class="kb-badge">{update.kb_article_id}</span>
      {/if}
      {#if update.severity}
        <span class="severity-badge {getSeverityClass(update.severity)}">
          {update.severity}
        </span>
      {/if}
      {#if update.is_mandatory}
        <span class="mandatory-badge">Mandatory</span>
      {/if}
      {#if update.size_bytes}
        <span class="size-badge">{formatBytes(update.size_bytes)}</span>
      {/if}

      {#if isDownloading}
        <span class="status-pill working">
          <span class="mini-spinner"></span> Downloading...
        </span>
      {:else if isInstalling}
        <span class="status-pill working">
          <span class="mini-spinner"></span> Installing...
        </span>
      {:else if isQueued}
        <span class="status-pill queued">
          {#if queuedType === "download"}
            Queued (Download)
          {:else if queuedType === "install"}
            Queued (Install)
          {:else}
            Queued (Download & Install)
          {/if}
        </span>
      {:else if update.is_downloaded}
        {#if rebootRequired}
          <span class="status-pill restart-pending">Restart Pending</span>
        {:else}
          <span class="status-pill ready">Ready to Install</span>
        {/if}
      {:else}
        <span class="status-pill pending">Pending Download</span>
      {/if}
    </div>

    <div class="update-title" title={update.title}>{update.title}</div>

    {#if update.description && !isExpanded}
      <div class="update-desc line-clamp">{update.description}</div>
    {/if}

    {#if isExpanded}
      <UpdateCardDetails {update} />
    {/if}

    <div class="card-footer-row">
      <button
        class="btn-details-toggle"
        onclick={() => (isExpanded = !isExpanded)}
      >
        {isExpanded ? "Hide details" : "View details"}
      </button>
    </div>
  </div>

  <UpdateCardActions
    updateId={update.id}
    isDownloaded={update.is_downloaded}
    {isDownloading}
    {isInstalling}
    {progress}
    {isQueued}
    {rebootRequired}
    {onDownload}
    {onInstall}
    {onDownloadAndInstall}
    {onCancelQueue}
  />
</div>

<style>
  .update-card {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.45rem;
    transition: border-color 0.15s ease, background-color 0.15s ease, box-shadow 0.15s ease;
  }

  .update-card:hover {
    border-color: color-mix(in srgb, var(--color-primary) 60%, var(--color-border));
  }

  .update-card.selected {
    border-color: var(--color-primary);
    background-color: color-mix(in srgb, var(--color-primary) 8%, var(--color-surface));
  }

  .update-card.downloaded {
    border-left: 4px solid var(--color-primary);
  }

  .update-card.in-progress {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px var(--color-primary);
  }

  .update-card.queued {
    border-color: color-mix(in srgb, var(--color-warning) 50%, var(--color-border));
  }

  .card-left-col {
    display: flex;
    align-items: center;
    padding-top: 0.15rem;
  }

  .checkbox-container {
    display: inline-flex;
    position: relative;
    cursor: pointer;
    user-select: none;
  }

  .card-checkbox {
    cursor: pointer;
    width: 16px;
    height: 16px;
    accent-color: var(--color-primary);
  }

  .update-main {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
    min-width: 0;
  }

  .update-badge-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .kb-badge {
    font-size: 0.72rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.12rem 0.45rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    border: solid var(--width-border) var(--color-border);
  }

  .severity-badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.12rem 0.45rem;
    border-radius: 0.25rem;
  }

  .sev-critical {
    background-color: color-mix(in srgb, var(--color-failure) 25%, var(--color-surface));
    color: var(--color-failure);
    border: solid 1px color-mix(in srgb, var(--color-failure) 50%, transparent);
  }

  .sev-important {
    background-color: color-mix(in srgb, var(--color-secondary) 25%, var(--color-surface));
    color: var(--color-secondary);
    border: solid 1px color-mix(in srgb, var(--color-secondary) 50%, transparent);
  }

  .sev-moderate {
    background-color: color-mix(in srgb, var(--color-warning) 25%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px color-mix(in srgb, var(--color-warning) 50%, transparent);
  }

  .sev-low,
  .sev-other,
  .sev-none {
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .mandatory-badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.12rem 0.45rem;
    border-radius: 0.25rem;
    background-color: color-mix(in srgb, var(--color-failure) 18%, var(--color-surface));
    color: var(--color-failure);
    border: solid 1px color-mix(in srgb, var(--color-failure) 35%, transparent);
  }

  .size-badge {
    font-size: 0.7rem;
    padding: 0.12rem 0.4rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .status-pill {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.12rem 0.5rem;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .status-pill.ready {
    background-color: var(--color-primary);
    color: var(--color-base);
  }

  .status-pill.restart-pending {
    background-color: color-mix(in srgb, var(--color-warning) 25%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px var(--color-warning);
  }

  .status-pill.pending {
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .status-pill.working {
    background-color: color-mix(in srgb, var(--color-primary) 20%, var(--color-surface));
    color: var(--color-primary);
    border: solid 1px var(--color-primary);
  }

  .status-pill.queued {
    background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px color-mix(in srgb, var(--color-warning) 45%, transparent);
  }

  .mini-spinner {
    width: 10px;
    height: 10px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .update-title {
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--color-text);
    line-height: 1.35;
  }

  .update-desc {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    line-height: 1.4;
  }

  .line-clamp {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-footer-row {
    display: flex;
    align-items: center;
  }

  .btn-details-toggle {
    background: transparent;
    border: none;
    color: var(--color-link);
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .btn-details-toggle:hover {
    text-decoration: underline;
  }
</style>
