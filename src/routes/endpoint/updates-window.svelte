<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import type { UpdateCheckResult } from "./types";
  import UpdateCard from "./update-card.svelte";

  interface Props {
    updates: UpdateCheckResult | null;
    isChecking: boolean;
    downloadingId: string | null;
    installingId: string | null;
    operationStatus: string | null;
    errorMessage: string | null;
    isAdmin?: boolean;
    onCheck: () => void;
    onDownload: (id?: string) => void;
    onInstall: (id?: string) => void;
  }

  let {
    updates,
    isChecking,
    downloadingId,
    installingId,
    operationStatus,
    errorMessage,
    isAdmin = true,
    onCheck,
    onDownload,
    onInstall,
  }: Props = $props();

  let hasPendingDownloads = $derived(
    updates?.updates.some((u) => !u.is_downloaded) ?? false,
  );
  let hasReadyInstalls = $derived(
    updates?.updates.some((u) => u.is_downloaded) ?? false,
  );
  let isBusy = $derived(
    isChecking || downloadingId !== null || installingId !== null,
  );
</script>

<div class="updates-window">
  <div class="window-header">
    <div class="title-cluster">
      <div class="header-icon-wrap">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 21h5v-5" />
        </svg>
      </div>
      <h3 class="window-title">System Updates</h3>
      {#if updates}
        <span
          class="count-badge"
          class:badge-has-updates={updates.updates.length > 0}
          class:badge-clean={updates.updates.length === 0}
        >
          {updates.updates.length}
        </span>
      {/if}
    </div>

    <div class="header-actions">
      {#if hasPendingDownloads}
        <button
          class="btn-action-primary"
          onclick={() => onDownload()}
          disabled={isBusy}
          title="Download all available updates"
        >
          {downloadingId === "ALL" ? "Downloading..." : "Download All"}
        </button>
      {/if}
      {#if hasReadyInstalls}
        <button
          class="btn-action-accent"
          onclick={() => onInstall()}
          disabled={isBusy}
          title="Install all downloaded updates"
        >
          {installingId === "ALL" ? "Installing..." : "Install All"}
        </button>
      {/if}
      <button
        class="btn-window-action"
        onclick={onCheck}
        disabled={isBusy}
        title="Check for Windows updates"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class:spin={isChecking}
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <span>{isChecking ? "Checking..." : "Check"}</span>
      </button>
    </div>
  </div>

  {#if !isAdmin}
    <div class="admin-warning-banner" role="status">
      <span class="admin-warning-icon">
        <AlertTriangle size={16} />
      </span>
      <span>Running without administrator privileges. Windows updates may not download correctly.</span>
    </div>
  {/if}

  {#if updates?.reboot_required}
    <div class="reboot-banner" role="alert">
      <svg
        class="reboot-icon"
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"
        />
        <line x1="12" y1="9" x2="12" y2="13" />
        <line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
      <span>A system restart is required to finish installing updates.</span>
    </div>
  {/if}

  {#if operationStatus}
    <div class="status-banner">
      <span>{operationStatus}</span>
    </div>
  {/if}

  {#if errorMessage}
    <div class="error-banner">
      <span>{errorMessage}</span>
      <button class="btn-retry" onclick={onCheck}>Retry</button>
    </div>
  {/if}

  <div class="window-content">
    {#if isChecking && !updates}
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Searching Windows Update catalog...</span>
      </div>
    {:else if updates && updates.updates.length === 0}
      <div class="clean-state">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="36"
          height="36"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--color-success)"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"
          />
          <path d="m9 12 2 2 4-4" />
        </svg>
        <span class="clean-title">System is up to date</span>
        <span class="clean-sub">Last checked: {updates.last_checked}</span>
      </div>
    {:else if updates && updates.updates.length > 0}
      <div class="updates-list">
        {#each updates.updates as update}
          <UpdateCard
            {update}
            {isBusy}
            isDownloading={downloadingId === update.id || downloadingId === "ALL"}
            isInstalling={installingId === update.id || installingId === "ALL"}
            onDownload={(id) => onDownload(id)}
            onInstall={(id) => onInstall(id)}
          />
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <span>Click "Check" to search for system updates.</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .updates-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .window-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .title-cluster {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-icon-wrap {
    color: var(--color-primary);
    display: flex;
    align-items: center;
  }

  .window-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .count-badge {
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
  }

  .badge-has-updates {
    background-color: var(--color-secondary);
    color: var(--color-base);
  }

  .badge-clean {
    background-color: var(--color-success);
    color: #1c1917;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .btn-action-primary,
  .btn-action-accent,
  .btn-window-action {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.28rem 0.65rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-action-primary {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-action-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-action-accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-action-accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-window-action {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-window-action:hover:not(:disabled) {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
  }

  .btn-action-primary:disabled,
  .btn-action-accent:disabled,
  .btn-window-action:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .admin-warning-banner,
  .reboot-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.55rem 1rem;
    background-color: color-mix(in srgb, #b57614 12%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, #b57614 35%, var(--color-border));
    color: #7c4a03;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .admin-warning-icon,
  .reboot-icon {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    color: #b57614;
  }

  @media (prefers-color-scheme: dark) {
    .admin-warning-banner,
    .reboot-banner {
      background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
      border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-warning) 45%, transparent);
      color: #fabd2f;
    }

    .admin-warning-icon,
    .reboot-icon {
      color: #fabd2f;
    }
  }

  .status-banner {
    padding: 0.45rem 1rem;
    background-color: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-primary) 40%, transparent);
    color: var(--color-primary);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-failure) 20%, var(--color-surface));
    border-bottom: solid var(--width-border) var(--color-failure);
    color: var(--color-failure);
    font-size: 0.8rem;
    font-weight: 600;
  }

  .btn-retry {
    background-color: transparent;
    border: solid 1px var(--color-failure);
    color: var(--color-failure);
    border-radius: 0.25rem;
    padding: 0.15rem 0.45rem;
    font-size: 0.72rem;
    font-weight: 700;
    cursor: pointer;
  }

  .btn-retry:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .window-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.75rem;
    background-color: var(--color-base);
  }

  .loading-state,
  .clean-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    height: 100%;
    min-height: 120px;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }

  .clean-title {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .clean-sub {
    font-size: 0.75rem;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--color-surface);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .updates-list {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }
</style>
