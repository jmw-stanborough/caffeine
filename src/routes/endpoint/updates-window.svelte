<script lang="ts">
  import { endpointStore, type EndpointStore } from "./endpoint.svelte";
  import type {
    UpdateCheckResult,
    UpdateFilterCategory,
    UpdateOperationType,
    UpdateSortOption,
  } from "./types";
  import { filterAndSortUpdates } from "./updates/filter";
  import UpdatesAdminBanner from "./updates/updates-admin-banner.svelte";
  import UpdatesBatchBar from "./updates/updates-batch-bar.svelte";
  import UpdatesEmptyState from "./updates/updates-empty-state.svelte";
  import UpdatesErrorBanner from "./updates/updates-error-banner.svelte";
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

  function dispatchOrQueue(
    type: UpdateOperationType,
    target: string | string[] | undefined,
    propAction?: (target?: string | string[]) => void,
    storeAction?: (target?: string | string[]) => Promise<unknown> | void,
  ) {
    if (isBusy && typeof target === "string") {
      const updateItem = effectiveUpdates?.updates.find((u) => u.id === target);
      store.queueOperation({
        id: target,
        title: updateItem?.title ?? target,
        type,
      });
      return;
    }

    if (propAction) {
      propAction(target);
    } else if (storeAction) {
      void storeAction(target);
    }
  }

  function handleDownload(target?: string | string[]) {
    dispatchOrQueue("download", target, propOnDownload, (t) => store.downloadUpdates(t));
  }

  function handleInstall(target?: string | string[]) {
    if (isRebootRequired) return;
    dispatchOrQueue("install", target, propOnInstall, (t) => store.installUpdates(t));
  }

  function handleDownloadAndInstall(target?: string | string[]) {
    if (isRebootRequired) return;
    dispatchOrQueue("download_and_install", target, propOnDownloadAndInstall, (t) =>
      store.downloadAndInstallUpdates(t),
    );
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

<div class="updates-module">
  <UpdatesHeader
    updates={effectiveUpdates}
    {isRebootRequired}
    {hasPendingDownloads}
    {hasReadyInstalls}
    {isBusy}
    downloadingId={effectiveDownloadingId}
    installingId={effectiveInstallingId}
    onRestart={() => (showRestartModal = true)}
    onDownloadAndInstall={() => handleDownloadAndInstall()}
    onDownload={() => handleDownload()}
    onInstall={() => handleInstall()}
  />

  {#if isRebootRequired}
    <UpdatesRebootBanner onRestart={() => (showRestartModal = true)} />
  {/if}

  {#if !effectiveIsAdmin}
    <UpdatesAdminBanner />
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
    <UpdatesErrorBanner message={effectiveErrorMessage} onRetry={handleCheck} />
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

  <div class="updates-content">
    {#if effectiveIsChecking}
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
  .updates-module {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    position: relative;
  }

  .updates-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-height: 0;
  }
</style>
