<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import { endpointStore, type EndpointStore } from "./endpoint.svelte";
  import type {
    UpdateCheckResult,
    UpdateFilterCategory,
    UpdateSortOption,
  } from "./types";
  import { filterAndSortUpdates } from "./updates/filter";
  import UpdatesBatchBar from "./updates/updates-batch-bar.svelte";
  import UpdatesEmptyState from "./updates/updates-empty-state.svelte";
  import UpdatesFilterBar from "./updates/updates-filter-bar.svelte";
  import UpdatesHeader from "./updates/updates-header.svelte";
  import UpdatesListView from "./updates/updates-list-view.svelte";
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

  let filteredUpdates = $derived(
    filterAndSortUpdates(effectiveUpdates?.updates, activeFilter, searchQuery, sortOption),
  );

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

  function confirmRestart() {
    showRestartModal = false;
    if (propOnRestart) {
      propOnRestart();
    } else {
      void store.restartSystem();
    }
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
    onRestart={() => (showRestartModal = true)}
    onDownloadAndInstall={() => handleDownloadAndInstall()}
    onDownload={() => handleDownload()}
    onInstall={() => handleInstall()}
    onCheck={handleCheck}
  />

  {#if isRebootRequired}
    <UpdatesRebootBanner onRestart={() => (showRestartModal = true)} />
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
      <UpdatesEmptyState state="loading" onCheck={handleCheck} />
    {:else if effectiveUpdates && effectiveUpdates.updates.length === 0}
      <UpdatesEmptyState
        state="clean"
        lastChecked={effectiveUpdates.last_checked}
        onCheck={handleCheck}
      />
    {:else if effectiveUpdates && filteredUpdates.length === 0}
      <UpdatesEmptyState
        state="no-match"
        onCheck={handleCheck}
        onClearFilters={() => {
          activeFilter = "all";
          searchQuery = "";
        }}
      />
    {:else if effectiveUpdates && filteredUpdates.length > 0}
      <UpdatesListView
        updates={filteredUpdates}
        {selectedIds}
        {allFilteredSelected}
        {store}
        {isBusy}
        {isRebootRequired}
        onToggleSelectAll={toggleSelectAll}
        onToggleSelectOne={toggleSelectOne}
        onDownload={handleDownload}
        onInstall={handleInstall}
        onDownloadAndInstall={handleDownloadAndInstall}
        onCancelQueue={(id) => store.cancelQueuedOperation(id)}
      />
    {:else}
      <UpdatesEmptyState state="no-data" onCheck={handleCheck} />
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
</style>
