<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { formatBytes } from "./metrics";
  import type { SystemUpdateItem, UpdateOperationType } from "./types";

  interface Props {
    update: SystemUpdateItem;
    isSelected: boolean;
    isDownloading: boolean;
    isInstalling: boolean;
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
    isQueued,
    queuedType,
    isBusy,
    rebootRequired,
    onToggleSelect,
    onDownload,
    onInstall,
    onDownloadAndInstall,
    onCancelQueue,
  }: Props = $props();

  let isExpanded = $state(false);
  let copiedId = $state(false);

  function getSeverityClass(severity?: string): string {
    if (!severity) return "sev-none";
    const s = severity.toLowerCase();
    if (s.includes("critical")) return "sev-critical";
    if (s.includes("important")) return "sev-important";
    if (s.includes("moderate")) return "sev-moderate";
    if (s.includes("low")) return "sev-low";
    return "sev-other";
  }

  async function handleOpenSupport(url: string) {
    try {
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  async function copyUpdateId(id: string) {
    try {
      await navigator.clipboard.writeText(id);
      copiedId = true;
      setTimeout(() => {
        copiedId = false;
      }, 2000);
    } catch (err) {
      console.warn("Failed to copy update ID:", err);
    }
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
      <div class="expanded-details">
        {#if update.description}
          <div class="full-desc">{update.description}</div>
        {/if}

        {#if update.categories.length > 0}
          <div class="categories-row">
            <span class="detail-label">Categories:</span>
            <div class="category-tags">
              {#each update.categories as cat}
                <span class="cat-pill">{cat}</span>
              {/each}
            </div>
          </div>
        {/if}

        <div class="meta-row">
          <span class="detail-label">Update ID:</span>
          <code class="update-id-code">{update.id}</code>
          <button
            class="btn-copy-id"
            onclick={() => copyUpdateId(update.id)}
            title="Copy Update GUID"
          >
            {copiedId ? "Copied!" : "Copy"}
          </button>
        </div>

        {#if update.support_url}
          <div class="meta-row">
            <span class="detail-label">Support Documentation:</span>
            <button
              class="support-link-btn"
              onclick={() => handleOpenSupport(update.support_url!)}
              title={update.support_url}
            >
              Open Microsoft Support Article &rarr;
            </button>
          </div>
        {/if}
      </div>
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

  <div class="update-card-actions">
    {#if isDownloading || isInstalling}
      <div class="btn-working-indicator">
        <span class="mini-spinner"></span>
        <span>{isDownloading ? "Downloading..." : "Installing..."}</span>
      </div>
    {:else if isQueued}
      <button
        class="btn-card-action cancel-queue"
        onclick={() => onCancelQueue(update.id)}
        title="Remove this update from the queue"
      >
        Cancel Queued
      </button>
    {:else if !update.is_downloaded}
      <div class="action-btn-group">
        <button
          class="btn-card-action hero"
          onclick={() => onDownloadAndInstall(update.id)}
          disabled={rebootRequired}
          title={rebootRequired
            ? "A system restart is required before updates can be installed."
            : "Download and immediately install this update"}
        >
          Download & Install
        </button>
        <button
          class="btn-card-action secondary"
          onclick={() => onDownload(update.id)}
          title="Download update package"
        >
          Download
        </button>
      </div>
    {:else if rebootRequired}
      <div class="action-btn-group">
        <button
          class="btn-card-action disabled-reboot"
          disabled
          title="A system restart is required before this update can be installed."
        >
          Restart Required
        </button>
      </div>
    {:else}
      <div class="action-btn-group">
        <button
          class="btn-card-action accent"
          onclick={() => onInstall(update.id)}
          title="Install this downloaded update"
        >
          Install Now
        </button>
      </div>
    {/if}
  </div>
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
    to { transform: rotate(360deg); }
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

  .expanded-details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    margin-top: 0.35rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .full-desc {
    font-size: 0.8rem;
    color: var(--color-text);
    line-height: 1.45;
  }

  .categories-row,
  .meta-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
    font-size: 0.76rem;
  }

  .detail-label {
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .category-tags {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .cat-pill {
    font-size: 0.68rem;
    padding: 0.08rem 0.35rem;
    border-radius: 0.2rem;
    background-color: var(--color-surface);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .update-id-code {
    font-size: 0.72rem;
    font-family: monospace;
    background-color: var(--color-surface);
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-copy-id {
    background: transparent;
    border: solid 1px var(--color-border);
    border-radius: 0.2rem;
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
    color: var(--color-text);
  }

  .btn-copy-id:hover {
    border-color: var(--color-primary);
    background-color: var(--color-surface);
  }

  .support-link-btn {
    background: transparent;
    border: none;
    color: var(--color-link);
    cursor: pointer;
    font-size: 0.76rem;
    font-weight: 600;
    padding: 0;
    text-decoration: underline;
  }

  .support-link-btn:hover {
    color: var(--color-link-hover);
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

  .update-card-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .action-btn-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 140px;
  }

  .btn-card-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0.38rem 0.8rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .btn-card-action.hero {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-card-action.hero:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-card-action.secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-card-action.secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover);
  }

  .btn-card-action.accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-card-action.accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-card-action.cancel-queue {
    background-color: transparent;
    border: solid 1px var(--color-warning);
    color: var(--color-warning);
    font-size: 0.74rem;
  }

  .btn-card-action.cancel-queue:hover {
    background-color: var(--color-warning);
    color: var(--color-base);
  }

  .btn-card-action.disabled-reboot {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-warning);
    color: var(--color-warning);
    opacity: 0.8;
    cursor: not-allowed;
  }

  .btn-card-action:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .btn-working-indicator {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.38rem 0.8rem;
    border-radius: 0.3rem;
    background-color: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
    border: solid 1px var(--color-primary);
    color: var(--color-primary);
    font-size: 0.78rem;
    font-weight: 600;
  }
</style>
