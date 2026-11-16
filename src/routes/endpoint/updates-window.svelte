<script lang="ts">
  import { endpointStore, type EndpointStore } from "./endpoint.svelte";
  import type {
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
  }

  let { store = endpointStore }: Props = $props();

  let isRebootRequired = $derived(store.updates?.reboot_required ?? false);

  let hasPendingDownloads = $derived(
    store.updates?.updates.some((u) => !u.is_downloaded) ?? false,
  );
  let hasReadyInstalls = $derived(
    store.updates?.updates.some((u) => u.is_downloaded) ?? false,
  );

  let pendingDownloadBytes = $derived(
    store.updates?.updates
      .filter((u) => !u.is_downloaded)
      .reduce((sum, u) => sum + (u.size_bytes ?? 0), 0) ?? 0,
  );

  let mandatoryCount = $derived(
    store.updates?.updates.filter((u) => u.is_mandatory).length ?? 0,
  );

  let readyToInstallCount = $derived(
    store.updates?.updates.filter((u) => u.is_downloaded).length ?? 0,
  );

  let pendingDownloadCount = $derived(
    store.updates?.updates.filter((u) => !u.is_downloaded).length ?? 0,
  );

  let isBusy = $derived(
    store.updatesChecking || store.downloadingId !== null || store.installingId !== null,
  );

  let activeFilter = $state<UpdateFilterCategory>("all");
  let sortOption = $state<UpdateSortOption>("severity");
  let searchQuery = $state("");
  let selectedIds = $state<Set<string>>(new Set());
  let showRestartModal = $state(false);

  let filteredUpdates = $derived(
    filterAndSortUpdates(store.updates?.updates, activeFilter, searchQuery, sortOption),
  );

  let allFilteredSelected = $derived(
    filteredUpdates.length > 0 &&
      filteredUpdates.every((u) => selectedIds.has(u.id)),
  );

  let selectedUpdatesList = $derived(
    store.updates?.updates.filter((u) => selectedIds.has(u.id)) ?? [],
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
    void store.checkForUpdates();
  }

  function dispatchOrQueue(
    type: UpdateOperationType,
    target: string | string[] | undefined,
    storeAction: (target?: string | string[]) => Promise<unknown> | void,
  ) {
    if (isBusy && typeof target === "string") {
      const updateItem = store.updates?.updates.find((u) => u.id === target);
      store.queueOperation({
        id: target,
        title: updateItem?.title ?? target,
        type,
      });
      return;
    }

    void storeAction(target);
  }

  function handleDownload(target?: string | string[]) {
    dispatchOrQueue("download", target, (t) => store.downloadUpdates(t));
  }

  function handleInstall(target?: string | string[]) {
    if (isRebootRequired) return;
    dispatchOrQueue("install", target, (t) => store.installUpdates(t));
  }

  function handleDownloadAndInstall(target?: string | string[]) {
    if (isRebootRequired) return;
    dispatchOrQueue("download_and_install", target, (t) => store.downloadAndInstallUpdates(t));
  }

  function confirmRestart() {
    showRestartModal = false;
    void store.restartSystem();
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
    updates={store.updates}
    {isRebootRequired}
    {hasPendingDownloads}
    {hasReadyInstalls}
    {isBusy}
    downloadingId={store.downloadingId}
    installingId={store.installingId}
    onRestart={() => (showRestartModal = true)}
    onDownloadAndInstall={() => handleDownloadAndInstall()}
    onDownload={() => handleDownload()}
    onInstall={() => handleInstall()}
  />

  {#if isRebootRequired}
    <UpdatesRebootBanner onRestart={() => (showRestartModal = true)} />
  {/if}

  {#if !store.isAdmin}
    <UpdatesAdminBanner />
  {/if}

  {#if store.operationStatus}
    <UpdatesStatusBanner
      operationStatus={store.operationStatus}
      updateProgress={store.updateProgress}
      queueLength={store.operationQueue.length}
      onClearQueue={() => store.clearQueue()}
    />
  {/if}

  {#if store.updatesError}
    <UpdatesErrorBanner message={store.updatesError} onRetry={handleCheck} />
  {/if}

  {#if store.updates && store.updates.updates.length > 0}
    <UpdatesStatsBar
      totalUpdates={store.updates.updates.length}
      {pendingDownloadCount}
      {pendingDownloadBytes}
      {readyToInstallCount}
      {mandatoryCount}
      lastChecked={store.updates.last_checked}
    />

    <UpdatesFilterBar
      {activeFilter}
      onFilterChange={(cat) => (activeFilter = cat)}
      totalCount={store.updates.updates.length}
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
    {#if store.updatesChecking}
      <UpdatesEmptyState state="loading" onCheck={handleCheck} />
    {:else if store.updates && store.updates.updates.length === 0}
      <UpdatesEmptyState
        state="clean"
        lastChecked={store.updates.last_checked}
        onCheck={handleCheck}
      />
    {:else if store.updates && filteredUpdates.length === 0}
      <UpdatesEmptyState
        state="no-match"
        onCheck={handleCheck}
        onClearFilters={() => {
          activeFilter = "all";
          searchQuery = "";
        }}
      />
    {:else if store.updates && filteredUpdates.length > 0}
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
