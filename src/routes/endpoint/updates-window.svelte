<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import { endpointStore, type EndpointStore } from "./endpoint.svelte";
  import type {
    UpdateCheckResult,
    UpdateFilterCategory,
    UpdateSortOption,
  } from "./types";
  import UpdateCard from "./update-card.svelte";
  import UpdatesBatchBar from "./updates/updates-batch-bar.svelte";
  import UpdatesFilterBar from "./updates/updates-filter-bar.svelte";
  import UpdatesHeader from "./updates/updates-header.svelte";
  import UpdatesRebootBanner from "./updates/updates-reboot-banner.svelte";
  import UpdatesRestartModal from "./updates/updates-restart-modal.svelte";
  import UpdatesStatsBar from "./updates/updates-stats-bar.svelte";
  import UpdatesStatusBanner from "./updates/updates-status-banner.svelte";

  interface Props {
    store?: EndpointStore;
    updates?: UpdateCheckResult | null;
    isChecking?: boolean;
    downloadingId?: string | null;
    installingId?: string | null;
    operationStatus?: string | null;
    errorMessage?: string | null;
    isAdmin?: boolean;
    onCheck?: () => void;
    onDownload?: (target?: string | string[]) => void;
    onInstall?: (target?: string | string[]) => void;
    onDownloadAndInstall?: (target?: string | string[]) => void;
    onRestart?: () => void;
  }

  let {
    store = endpointStore,
    updates: propUpdates,
    isChecking: propIsChecking,
    downloadingId: propDownloadingId,
    installingId: propInstallingId,
    operationStatus: propOperationStatus,
    errorMessage: propErrorMessage,
    isAdmin: propIsAdmin,
    onCheck: propOnCheck,
    onDownload: propOnDownload,
    onInstall: propOnInstall,
    onDownloadAndInstall: propOnDownloadAndInstall,
    onRestart: propOnRestart,
  }: Props = $props();

  let effectiveUpdates = $derived(propUpdates !== undefined ? propUpdates : store.updates);
  let effectiveIsChecking = $derived(
    propIsChecking !== undefined ? propIsChecking : store.updatesChecking,
  );
  let effectiveDownloadingId = $derived(
    propDownloadingId !== undefined ? propDownloadingId : store.downloadingId,
  );
  let effectiveInstallingId = $derived(
    propInstallingId !== undefined ? propInstallingId : store.installingId,
  );
  let effectiveOperationStatus = $derived(
    propOperationStatus !== undefined ? propOperationStatus : store.operationStatus,
  );
  let effectiveErrorMessage = $derived(
    propErrorMessage !== undefined ? propErrorMessage : store.updatesError,
  );
  let effectiveIsAdmin = $derived(
    propIsAdmin !== undefined ? propIsAdmin : store.isAdmin,
  );

  let isRebootRequired = $derived(effectiveUpdates?.reboot_required ?? false);

  let hasPendingDownloads = $derived(
    effectiveUpdates?.updates.some((u) => !u.is_downloaded) ?? false,
  );
  let hasReadyInstalls = $derived(
    effectiveUpdates?.updates.some((u) => u.is_downloaded) ?? false,
  );

  let pendingDownloadBytes = $derived(
    effectiveUpdates?.updates
      .filter((u) => !u.is_downloaded)
      .reduce((sum, u) => sum + (u.size_bytes ?? 0), 0) ?? 0,
  );

  let mandatoryCount = $derived(
    effectiveUpdates?.updates.filter((u) => u.is_mandatory).length ?? 0,
  );

  let readyToInstallCount = $derived(
    effectiveUpdates?.updates.filter((u) => u.is_downloaded).length ?? 0,
  );

  let pendingDownloadCount = $derived(
    effectiveUpdates?.updates.filter((u) => !u.is_downloaded).length ?? 0,
  );

  let isBusy = $derived(
    effectiveIsChecking || effectiveDownloadingId !== null || effectiveInstallingId !== null,
  );

  let activeFilter = $state<UpdateFilterCategory>("all");
  let sortOption = $state<UpdateSortOption>("severity");
  let searchQuery = $state("");
  let selectedIds = $state<Set<string>>(new Set());
  let showRestartModal = $state(false);

  function getSeverityWeight(severity?: string): number {
    if (!severity) return 0;
    const s = severity.toLowerCase();
    if (s.includes("critical")) return 4;
    if (s.includes("important")) return 3;
    if (s.includes("moderate")) return 2;
    if (s.includes("low")) return 1;
    return 0;
  }

  let filteredUpdates = $derived.by(() => {
    if (!effectiveUpdates?.updates) return [];

    let list = [...effectiveUpdates.updates];

    if (activeFilter === "pending_download") {
      list = list.filter((u) => !u.is_downloaded);
    } else if (activeFilter === "ready_to_install") {
      list = list.filter((u) => u.is_downloaded);
    } else if (activeFilter === "mandatory") {
      list = list.filter((u) => u.is_mandatory);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      list = list.filter(
        (u) =>
          u.title.toLowerCase().includes(q) ||
          (u.kb_article_id && u.kb_article_id.toLowerCase().includes(q)) ||
          (u.description && u.description.toLowerCase().includes(q)) ||
          u.categories.some((c) => c.toLowerCase().includes(q)),
      );
    }

    list.sort((a, b) => {
      if (sortOption === "severity") {
        const diff = getSeverityWeight(b.severity) - getSeverityWeight(a.severity);
        if (diff !== 0) return diff;
        return (b.size_bytes ?? 0) - (a.size_bytes ?? 0);
      }
      if (sortOption === "size") {
        return (b.size_bytes ?? 0) - (a.size_bytes ?? 0);
      }
      if (sortOption === "title") {
        return a.title.localeCompare(b.title);
      }
      if (sortOption === "kb") {
        return (a.kb_article_id ?? "").localeCompare(b.kb_article_id ?? "");
      }
      return 0;
    });

    return list;
  });

  let allFilteredSelected = $derived(
    filteredUpdates.length > 0 &&
      filteredUpdates.every((u) => selectedIds.has(u.id)),
  );

  let selectedUpdatesList = $derived(
    effectiveUpdates?.updates.filter((u) => selectedIds.has(u.id)) ?? [],
  );

  let selectedTotalBytes = $derived(
    selectedUpdatesList.reduce((sum, u) => sum + (u.size_bytes ?? 0), 0),
  );

  let selectedPendingCount = $derived(
    selectedUpdatesList.filter((u) => !u.is_downloaded).length,
  );

  let selectedReadyCount = $derived(
    selectedUpdatesList.filter((u) => u.is_downloaded).length,
  );

  function toggleSelectAll() {
    if (allFilteredSelected) {
      selectedIds = new Set();
    } else {
      const next = new Set(selectedIds);
      for (const u of filteredUpdates) {
        next.add(u.id);
      }
      selectedIds = next;
    }
  }

  function toggleSelectOne(id: string) {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedIds = next;
  }

  function clearSelection() {
    selectedIds = new Set();
  }

  function handleCheck() {
    if (propOnCheck) {
      propOnCheck();
    } else {
      void store.checkForUpdates();
    }
  }

  function handleDownload(target?: string | string[]) {
    if (isBusy && typeof target === "string") {
      const updateItem = effectiveUpdates?.updates.find((u) => u.id === target);
      store.queueOperation({
        id: target,
        title: updateItem?.title ?? target,
        type: "download",
      });
      return;
    }

    if (propOnDownload) {
      propOnDownload(target);
    } else {
      void store.downloadUpdates(target);
    }
  }

  function handleInstall(target?: string | string[]) {
    if (isRebootRequired) return;

    if (isBusy && typeof target === "string") {
      const updateItem = effectiveUpdates?.updates.find((u) => u.id === target);
      store.queueOperation({
        id: target,
        title: updateItem?.title ?? target,
        type: "install",
      });
      return;
    }

    if (propOnInstall) {
      propOnInstall(target);
    } else {
      void store.installUpdates(target);
    }
  }

  function handleDownloadAndInstall(target?: string | string[]) {
    if (isRebootRequired) return;

    if (isBusy && typeof target === "string") {
      const updateItem = effectiveUpdates?.updates.find((u) => u.id === target);
      store.queueOperation({
        id: target,
        title: updateItem?.title ?? target,
        type: "download_and_install",
      });
      return;
    }

    if (propOnDownloadAndInstall) {
      propOnDownloadAndInstall(target);
    } else {
      void store.downloadAndInstallUpdates(target);
    }
  }

  function handleRestartClick() {
    showRestartModal = true;
  }

  function confirmRestart() {
    showRestartModal = false;
    if (propOnRestart) {
      propOnRestart();
    } else {
      void store.restartSystem();
    }
  }

  function handleCancelQueue(id: string) {
    store.cancelQueuedOperation(id);
  }

  function handleBatchDownload() {
    if (selectedIds.size === 0) return;
    handleDownload(Array.from(selectedIds));
    clearSelection();
  }

  function handleBatchInstall() {
    if (selectedIds.size === 0 || isRebootRequired) return;
    handleInstall(Array.from(selectedIds));
    clearSelection();
  }

  function handleBatchDownloadAndInstall() {
    if (selectedIds.size === 0 || isRebootRequired) return;
    handleDownloadAndInstall(Array.from(selectedIds));
    clearSelection();
  }
</script>

<div class="updates-window">
  <UpdatesHeader
    updates={effectiveUpdates}
    {isRebootRequired}
    {hasPendingDownloads}
    {hasReadyInstalls}
    {isBusy}
    downloadingId={effectiveDownloadingId}
    installingId={effectiveInstallingId}
    isChecking={effectiveIsChecking}
    onRestart={handleRestartClick}
    onDownloadAndInstall={() => handleDownloadAndInstall()}
    onDownload={() => handleDownload()}
    onInstall={() => handleInstall()}
    onCheck={handleCheck}
  />

  {#if isRebootRequired}
    <UpdatesRebootBanner onRestart={handleRestartClick} />
  {/if}

  {#if !effectiveIsAdmin}
    <div class="admin-warning-banner" role="status">
      <span class="admin-warning-icon">
        <AlertTriangle size={16} />
      </span>
      <span>
        Running without administrator privileges. Elevated permissions may be required to download or install updates.
      </span>
    </div>
  {/if}

  {#if effectiveOperationStatus}
    <UpdatesStatusBanner
      operationStatus={effectiveOperationStatus}
      updateProgress={store.updateProgress}
      queueLength={store.operationQueue.length}
      onClearQueue={() => store.clearQueue()}
    />
  {/if}

  {#if effectiveErrorMessage}
    <div class="error-banner">
      <span>{effectiveErrorMessage}</span>
      <button class="btn-retry" onclick={handleCheck}>Retry</button>
    </div>
  {/if}

  {#if effectiveUpdates && effectiveUpdates.updates.length > 0}
    <UpdatesStatsBar
      totalUpdates={effectiveUpdates.updates.length}
      {pendingDownloadCount}
      {pendingDownloadBytes}
      {readyToInstallCount}
      {mandatoryCount}
      lastChecked={effectiveUpdates.last_checked}
    />

    <UpdatesFilterBar
      {activeFilter}
      onFilterChange={(cat) => (activeFilter = cat)}
      totalCount={effectiveUpdates.updates.length}
      {pendingDownloadCount}
      {readyToInstallCount}
      {mandatoryCount}
      {searchQuery}
      onSearchChange={(q) => (searchQuery = q)}
      {sortOption}
      onSortChange={(opt) => (sortOption = opt)}
    />

    {#if selectedIds.size > 0}
      <UpdatesBatchBar
        selectedCount={selectedIds.size}
        selectedBytes={selectedTotalBytes}
        {selectedPendingCount}
        {selectedReadyCount}
        {isBusy}
        {isRebootRequired}
        onBatchDownloadAndInstall={handleBatchDownloadAndInstall}
        onBatchDownload={handleBatchDownload}
        onBatchInstall={handleBatchInstall}
        onClearSelection={clearSelection}
      />
    {/if}
  {/if}

  <div class="window-content">
    {#if effectiveIsChecking && !effectiveUpdates}
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Searching Windows Update catalog...</span>
      </div>
    {:else if effectiveUpdates && effectiveUpdates.updates.length === 0}
      <div class="clean-state">
        <div class="clean-icon-wrap">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="42"
            height="42"
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--color-success)"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" />
            <path d="m9 12 2 2 4-4" />
          </svg>
        </div>
        <span class="clean-title">System is Up to Date</span>
        <span class="clean-sub">No new updates are currently available.</span>
        <span class="clean-meta">Last checked: {effectiveUpdates.last_checked}</span>
        <button class="btn-check-again" onclick={handleCheck}>
          Check Again
        </button>
      </div>
    {:else if effectiveUpdates && filteredUpdates.length === 0}
      <div class="empty-state">
        <span>No updates match the selected filter or search criteria.</span>
        <button
          class="btn-clear-search-link"
          onclick={() => {
            activeFilter = "all";
            searchQuery = "";
          }}
        >
          Clear filters
        </button>
      </div>
    {:else if effectiveUpdates && filteredUpdates.length > 0}
      <div class="updates-list-header">
        <label class="select-all-label">
          <input
            type="checkbox"
            checked={allFilteredSelected}
            onchange={toggleSelectAll}
            class="card-checkbox"
          />
          <span>Select all shown ({filteredUpdates.length})</span>
        </label>
      </div>

      <div class="updates-list">
        {#each filteredUpdates as update (update.id)}
          <UpdateCard
            {update}
            isSelected={selectedIds.has(update.id)}
            isDownloading={store.isUpdateDownloading(update.id)}
            isInstalling={store.isUpdateInstalling(update.id)}
            progress={store.getUpdateProgress(update.id)}
            isQueued={store.isUpdateQueued(update.id)}
            queuedType={store.getQueuedOperation(update.id)?.type}
            {isBusy}
            rebootRequired={isRebootRequired}
            onToggleSelect={toggleSelectOne}
            onDownload={handleDownload}
            onInstall={handleInstall}
            onDownloadAndInstall={handleDownloadAndInstall}
            onCancelQueue={handleCancelQueue}
          />
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="36"
          height="36"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--color-text-muted)"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <span>No update data loaded.</span>
        <button class="btn-action-primary" onclick={handleCheck}>
          Check for Updates Now
        </button>
      </div>
    {/if}
  </div>
</div>

<UpdatesRestartModal
  open={showRestartModal}
  onClose={() => (showRestartModal = false)}
  onConfirm={confirmRestart}
/>

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

  .admin-warning-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-warning) 12%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-warning) 30%, transparent);
    color: var(--color-text);
    font-size: 0.78rem;
  }

  .admin-warning-icon {
    color: var(--color-warning);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.55rem 1rem;
    background-color: color-mix(in srgb, var(--color-failure) 14%, var(--color-surface));
    border-bottom: solid var(--width-border) var(--color-failure);
    color: var(--color-failure);
    font-size: 0.8rem;
  }

  .btn-retry {
    background: transparent;
    border: solid 1px var(--color-failure);
    color: var(--color-failure);
    font-size: 0.74rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .btn-retry:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .window-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-height: 0;
  }

  .updates-list-header {
    display: flex;
    align-items: center;
    padding: 0.25rem 0;
  }

  .select-all-label {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.76rem;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .card-checkbox {
    cursor: pointer;
    width: 15px;
    height: 15px;
    accent-color: var(--color-primary);
  }

  .updates-list {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .loading-state,
  .clean-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1.5rem;
    gap: 0.75rem;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    margin: auto 0;
  }

  .clean-title {
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .clean-sub {
    font-size: 0.82rem;
  }

  .clean-meta {
    font-size: 0.74rem;
    opacity: 0.75;
  }

  .btn-check-again {
    margin-top: 0.5rem;
    padding: 0.38rem 0.85rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-check-again:hover {
    border-color: var(--color-primary);
  }

  .btn-clear-search-link {
    background: transparent;
    border: none;
    color: var(--color-link);
    text-decoration: underline;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .btn-action-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.4rem 0.85rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
    cursor: pointer;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
