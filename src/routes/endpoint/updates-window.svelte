<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import Modal from "$lib/components/modal.svelte";
  import { endpointStore, type EndpointStore } from "./endpoint.svelte";
  import { formatBytes } from "./metrics";
  import type {
    SystemUpdateItem,
    UpdateCheckResult,
    UpdateFilterCategory,
    UpdateSortOption,
  } from "./types";
  import UpdateCard from "./update-card.svelte";

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
  <div class="window-header">
    <div class="title-cluster">
      <div class="header-icon-wrap">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
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

      {#if effectiveUpdates}
        <span
          class="count-badge"
          class:badge-has-updates={effectiveUpdates.updates.length > 0}
          class:badge-clean={effectiveUpdates.updates.length === 0}
        >
          {effectiveUpdates.updates.length} Available
        </span>
      {/if}

      {#if isRebootRequired}
        <span class="reboot-status-pill">
          <span class="pulse-dot"></span> Restart Pending
        </span>
      {/if}
    </div>

    <div class="header-actions">
      {#if isRebootRequired}
        <button
          class="btn-action-restart"
          onclick={handleRestartClick}
          title="Restart the system immediately to complete updates"
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
          >
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          <span>Restart Now</span>
        </button>
      {/if}

      {#if effectiveUpdates && effectiveUpdates.updates.length > 0}
        <button
          class="btn-action-primary"
          onclick={() => handleDownloadAndInstall()}
          disabled={isBusy || isRebootRequired}
          title={isRebootRequired
            ? "A system restart is required before updates can be installed."
            : "Download and install all available updates in one step"}
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
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          <span>
            {effectiveDownloadingId === "ALL" && effectiveInstallingId === "ALL"
              ? "Processing..."
              : "Download & Install All"}
          </span>
        </button>

        {#if hasPendingDownloads}
          <button
            class="btn-action-secondary"
            onclick={() => handleDownload()}
            disabled={isBusy}
            title="Download all pending update packages"
          >
            <span>{effectiveDownloadingId === "ALL" ? "Downloading..." : "Download All"}</span>
          </button>
        {/if}

        {#if hasReadyInstalls}
          <button
            class="btn-action-accent"
            onclick={() => handleInstall()}
            disabled={isBusy || isRebootRequired}
            title={isRebootRequired
              ? "A system restart is required before installing updates."
              : "Install all downloaded updates"}
          >
            <span>{effectiveInstallingId === "ALL" ? "Installing..." : "Install All"}</span>
          </button>
        {/if}
      {/if}

      <button
        class="btn-window-action"
        onclick={handleCheck}
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
          class:spin={effectiveIsChecking}
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <span>{effectiveIsChecking ? "Checking..." : "Check"}</span>
      </button>
    </div>
  </div>

  {#if isRebootRequired}
    <div class="reboot-hero-banner" role="alert">
      <div class="reboot-hero-left">
        <div class="reboot-hero-icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
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
        </div>
        <div class="reboot-hero-text">
          <span class="reboot-hero-title">System Restart Required</span>
          <span class="reboot-hero-desc">
            A restart is pending to complete installed updates. Additional update installations are
            temporarily paused until the system is restarted.
          </span>
        </div>
      </div>
      <button class="btn-hero-restart" onclick={handleRestartClick}>
        Restart Now
      </button>
    </div>
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
    <div class="status-banner">
      <div class="status-content">
        <span class="mini-spinner"></span>
        <div class="status-text-block">
          <div class="status-title-row">
            <span>{effectiveOperationStatus}</span>
            <span class="progress-percent-label">
              {store.updateProgress ? `${store.updateProgress.percent_complete}%` : ""}
            </span>
          </div>
          <div
            class="status-progress-track"
            role="progressbar"
            aria-valuenow={store.updateProgress?.percent_complete ?? 0}
            aria-valuemin={0}
            aria-valuemax={100}
          >
            <div
              class="status-progress-fill"
              class:indeterminate={!store.updateProgress || store.updateProgress.percent_complete === 0}
              style:width="{store.updateProgress ? Math.max(store.updateProgress.percent_complete, 4) : 100}%"
            ></div>
          </div>
          <div class="status-progress-meta">
            {#if store.updateProgress && store.updateProgress.total_count > 1}
              <span>Processing update {store.updateProgress.current_index + 1} of {store.updateProgress.total_count}</span>
            {:else if store.updateProgress}
              <span>{store.updateProgress.phase === 'download' ? 'Downloading package...' : 'Installing package...'}</span>
            {:else}
              <span>Processing...</span>
            {/if}
            {#if store.updateProgress && store.updateProgress.phase === 'download' && store.updateProgress.bytes_downloaded != null && store.updateProgress.total_bytes}
              <span>{formatBytes(store.updateProgress.bytes_downloaded)} / {formatBytes(store.updateProgress.total_bytes)}</span>
            {/if}
          </div>
        </div>
      </div>
      {#if store.operationQueue.length > 0}
        <div class="queue-info">
          <span class="queue-pill">{store.operationQueue.length} queued</span>
          <button class="btn-clear-queue" onclick={() => store.clearQueue()}>
            Clear Queue
          </button>
        </div>
      {/if}
    </div>
  {/if}

  {#if effectiveErrorMessage}
    <div class="error-banner">
      <span>{effectiveErrorMessage}</span>
      <button class="btn-retry" onclick={handleCheck}>Retry</button>
    </div>
  {/if}

  {#if effectiveUpdates && effectiveUpdates.updates.length > 0}
    <div class="stats-overview-bar">
      <div class="stat-cell">
        <span class="stat-value">{effectiveUpdates.updates.length}</span>
        <span class="stat-label">Total Updates</span>
      </div>
      <div class="stat-cell">
        <span class="stat-value">{pendingDownloadCount}</span>
        <span class="stat-label">Pending Download ({formatBytes(pendingDownloadBytes)})</span>
      </div>
      <div class="stat-cell">
        <span class="stat-value">{readyToInstallCount}</span>
        <span class="stat-label">Ready to Install</span>
      </div>
      {#if mandatoryCount > 0}
        <div class="stat-cell mandatory">
          <span class="stat-value">{mandatoryCount}</span>
          <span class="stat-label">Mandatory / Security</span>
        </div>
      {/if}
      <div class="stat-cell-meta">
        <span class="stat-label">Last Checked:</span>
        <span class="stat-meta-value">{effectiveUpdates.last_checked}</span>
      </div>
    </div>

    <div class="filter-controls-row">
      <div class="filter-tabs">
        <button
          class="filter-tab"
          class:active={activeFilter === "all"}
          onclick={() => (activeFilter = "all")}
        >
          All ({effectiveUpdates.updates.length})
        </button>
        <button
          class="filter-tab"
          class:active={activeFilter === "pending_download"}
          onclick={() => (activeFilter = "pending_download")}
        >
          Pending ({pendingDownloadCount})
        </button>
        <button
          class="filter-tab"
          class:active={activeFilter === "ready_to_install"}
          onclick={() => (activeFilter = "ready_to_install")}
        >
          Ready ({readyToInstallCount})
        </button>
        {#if mandatoryCount > 0}
          <button
            class="filter-tab"
            class:active={activeFilter === "mandatory"}
            onclick={() => (activeFilter = "mandatory")}
          >
            Mandatory ({mandatoryCount})
          </button>
        {/if}
      </div>

      <div class="search-and-sort">
        <div class="search-input-wrap">
          <svg
            class="search-icon"
            xmlns="http://www.w3.org/2000/svg"
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            class="search-input"
            placeholder="Search updates, KB, categories..."
            bind:value={searchQuery}
          />
          {#if searchQuery}
            <button class="btn-clear-search" onclick={() => (searchQuery = "")}>&times;</button>
          {/if}
        </div>

        <div class="sort-wrap">
          <label for="sort-select" class="sort-label">Sort:</label>
          <select id="sort-select" class="sort-select" bind:value={sortOption}>
            <option value="severity">Severity</option>
            <option value="size">Size</option>
            <option value="title">Title</option>
            <option value="kb">KB Number</option>
          </select>
        </div>
      </div>
    </div>

    {#if selectedIds.size > 0}
      <div class="batch-selection-bar">
        <div class="batch-info">
          <span class="batch-count">
            <strong>{selectedIds.size}</strong> update(s) selected
          </span>
          {#if selectedTotalBytes > 0}
            <span class="batch-size">({formatBytes(selectedTotalBytes)})</span>
          {/if}
        </div>

        <div class="batch-actions">
          <button
            class="btn-batch-primary"
            onclick={handleBatchDownloadAndInstall}
            disabled={isBusy || isRebootRequired}
            title={isRebootRequired
              ? "A system restart is required before installing updates."
              : "Download and install selected updates"}
          >
            Download & Install Selected ({selectedIds.size})
          </button>

          {#if selectedPendingCount > 0}
            <button
              class="btn-batch-secondary"
              onclick={handleBatchDownload}
              disabled={isBusy}
              title="Download selected pending updates"
            >
              Download Selected ({selectedPendingCount})
            </button>
          {/if}

          {#if selectedReadyCount > 0}
            <button
              class="btn-batch-accent"
              onclick={handleBatchInstall}
              disabled={isBusy || isRebootRequired}
              title={isRebootRequired
                ? "A system restart is required before installing updates."
                : "Install selected downloaded updates"}
            >
              Install Selected ({selectedReadyCount})
            </button>
          {/if}

          <button class="btn-batch-deselect" onclick={clearSelection}>
            Deselect All
          </button>
        </div>
      </div>
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

<Modal open={showRestartModal} onBackdropClose={() => (showRestartModal = false)}>
  <div class="restart-modal-content">
    <div class="modal-header-row">
      <div class="modal-icon-wrap warning">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
        </svg>
      </div>
      <div>
        <h3 class="modal-title">Restart System</h3>
        <p class="modal-subtitle">A system restart is pending to complete Windows updates.</p>
      </div>
    </div>

    <div class="modal-body-text">
      <p>
        Your computer will restart immediately. Please ensure all your work and open files in other
        applications are saved before continuing.
      </p>
      <p class="modal-note">
        After the computer reboots, Caffeine will automatically restore your monitoring session.
      </p>
    </div>

    <div class="modal-footer-actions">
      <button class="btn-modal-cancel" onclick={() => (showRestartModal = false)}>
        Cancel
      </button>
      <button class="btn-modal-confirm-restart" onclick={confirmRestart}>
        Restart Now
      </button>
    </div>
  </div>
</Modal>

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
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .title-cluster {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
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
    padding: 0.12rem 0.45rem;
    border-radius: 999px;
  }

  .badge-has-updates {
    background-color: var(--color-secondary);
    color: var(--color-base);
  }

  .badge-clean {
    background-color: var(--color-success);
    color: var(--color-tag-text-success, #ffffff);
  }

  .reboot-status-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.12rem 0.5rem;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--color-warning) 25%, var(--color-surface));
    color: var(--color-warning);
    border: solid 1px var(--color-warning);
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-warning);
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0% { transform: scale(0.95); opacity: 0.8; }
    50% { transform: scale(1.3); opacity: 1; }
    100% { transform: scale(0.95); opacity: 0.8; }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .btn-action-primary,
  .btn-action-secondary,
  .btn-action-accent,
  .btn-action-restart,
  .btn-window-action {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.7rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
    white-space: nowrap;
  }

  .btn-action-restart {
    background-color: color-mix(in srgb, var(--color-warning) 30%, var(--color-surface));
    border: solid 1px var(--color-warning);
    color: var(--color-warning);
  }

  .btn-action-restart:hover {
    background-color: var(--color-warning);
    color: var(--color-base);
  }

  .btn-action-primary {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-action-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-action-secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-action-secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover);
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
  .btn-action-secondary:disabled,
  .btn-action-accent:disabled,
  .btn-window-action:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .reboot-hero-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background-color: color-mix(in srgb, var(--color-warning) 16%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-warning) 40%, var(--color-border));
    gap: 1rem;
    flex-wrap: wrap;
  }

  .reboot-hero-left {
    display: flex;
    align-items: flex-start;
    gap: 0.65rem;
  }

  .reboot-hero-icon {
    color: var(--color-warning);
    display: flex;
    align-items: center;
    flex-shrink: 0;
    padding-top: 0.1rem;
  }

  .reboot-hero-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .reboot-hero-title {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--color-warning);
  }

  .reboot-hero-desc {
    font-size: 0.78rem;
    color: var(--color-text);
    line-height: 1.35;
  }

  .btn-hero-restart {
    background-color: var(--color-warning);
    border: solid 1px var(--color-warning);
    color: var(--color-base);
    padding: 0.4rem 0.9rem;
    border-radius: 0.3rem;
    font-size: 0.82rem;
    font-weight: 700;
    cursor: pointer;
    transition: filter 0.15s ease;
    white-space: nowrap;
  }

  .btn-hero-restart:hover {
    filter: brightness(1.15);
  }

  .admin-warning-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-warning) 12%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-warning) 30%, var(--color-border));
    color: var(--color-warning);
    font-size: 0.8rem;
    font-weight: 600;
  }

  .admin-warning-icon {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }

  .status-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.55rem 1rem;
    background-color: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
    border-bottom: solid var(--width-border) color-mix(in srgb, var(--color-primary) 40%, transparent);
    color: var(--color-primary);
    font-size: 0.8rem;
    font-weight: 500;
    gap: 0.75rem;
  }

  .status-content {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
  }

  .status-text-block {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    flex: 1;
    max-width: 550px;
  }

  .status-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .progress-percent-label {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .status-progress-track {
    width: 100%;
    height: 6px;
    background-color: color-mix(in srgb, var(--color-primary) 25%, transparent);
    border-radius: 3px;
    overflow: hidden;
  }

  .status-progress-fill {
    height: 100%;
    background-color: var(--color-primary);
    border-radius: 3px;
    transition: width 0.25s ease-out;
  }

  .status-progress-fill.indeterminate {
    background: linear-gradient(
      90deg,
      var(--color-primary) 0%,
      color-mix(in srgb, var(--color-primary) 40%, white) 50%,
      var(--color-primary) 100%
    );
    background-size: 200% 100%;
    animation: shimmer-progress 1.5s infinite linear;
  }

  @keyframes shimmer-progress {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .status-progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    opacity: 0.85;
    font-variant-numeric: tabular-nums;
  }

  .queue-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .queue-pill {
    font-size: 0.72rem;
    font-weight: 700;
    background-color: var(--color-surface);
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-clear-queue {
    background: transparent;
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
    font-size: 0.72rem;
    border-radius: 0.2rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
  }

  .btn-clear-queue:hover {
    color: var(--color-failure);
    border-color: var(--color-failure);
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

  .stats-overview-bar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.6rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-wrap: wrap;
  }

  .stat-cell {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
  }

  .stat-cell.mandatory .stat-value,
  .stat-cell.mandatory .stat-label {
    color: var(--color-failure);
  }

  .stat-value {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .stat-label {
    font-size: 0.76rem;
    color: var(--color-text-muted);
  }

  .stat-cell-meta {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.76rem;
  }

  .stat-meta-value {
    color: var(--color-text);
    font-weight: 600;
  }

  .filter-controls-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .filter-tabs {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .filter-tab {
    background: transparent;
    border: solid 1px transparent;
    border-radius: 0.3rem;
    padding: 0.25rem 0.6rem;
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .filter-tab:hover {
    color: var(--color-text);
    background-color: var(--color-surface-hover);
  }

  .filter-tab.active {
    background-color: var(--color-base);
    border-color: var(--color-border);
    color: var(--color-primary);
  }

  .search-and-sort {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 0.55rem;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .search-input {
    padding: 0.25rem 1.6rem 0.25rem 1.7rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.3rem;
    font-size: 0.76rem;
    color: var(--color-text);
    outline: none;
    width: 220px;
    transition: border-color 0.15s ease;
  }

  .search-input:focus {
    border-color: var(--color-primary);
  }

  .btn-clear-search {
    position: absolute;
    right: 0.4rem;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0;
  }

  .sort-wrap {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .sort-label {
    font-size: 0.76rem;
    color: var(--color-text-muted);
  }

  .sort-select {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.3rem;
    color: var(--color-text);
    padding: 0.25rem 0.5rem;
    font-size: 0.76rem;
    outline: none;
    cursor: pointer;
  }

  .sort-select:focus {
    border-color: var(--color-primary);
  }

  .batch-selection-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-primary) 12%, var(--color-surface));
    border-bottom: solid var(--width-border) var(--color-primary);
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .batch-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--color-text);
  }

  .batch-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .btn-batch-primary,
  .btn-batch-secondary,
  .btn-batch-accent,
  .btn-batch-deselect {
    padding: 0.25rem 0.6rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-batch-primary {
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
  }

  .btn-batch-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-batch-secondary {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-batch-secondary:hover:not(:disabled) {
    border-color: var(--color-primary);
  }

  .btn-batch-accent {
    background-color: var(--color-secondary);
    border: solid 1px var(--color-secondary);
    color: var(--color-base);
  }

  .btn-batch-accent:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-batch-deselect {
    background: transparent;
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
  }

  .btn-batch-deselect:hover {
    color: var(--color-text);
    background-color: var(--color-surface);
  }

  .btn-batch-primary:disabled,
  .btn-batch-accent:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .window-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.75rem;
    background-color: var(--color-base);
  }

  .updates-list-header {
    display: flex;
    align-items: center;
    padding: 0.2rem 0.5rem 0.5rem;
  }

  .select-all-label {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.76rem;
    color: var(--color-text-muted);
    cursor: pointer;
    user-select: none;
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
    gap: 0.65rem;
  }

  .loading-state,
  .clean-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    height: 100%;
    min-height: 160px;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    text-align: center;
  }

  .clean-icon-wrap {
    margin-bottom: 0.25rem;
  }

  .clean-title {
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .clean-sub {
    font-size: 0.82rem;
    color: var(--color-text-muted);
  }

  .clean-meta {
    font-size: 0.74rem;
    color: var(--color-text-muted);
  }

  .btn-check-again {
    margin-top: 0.4rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.35rem 0.8rem;
    border-radius: 0.3rem;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-check-again:hover {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover);
  }

  .btn-clear-search-link {
    background: transparent;
    border: none;
    color: var(--color-link);
    text-decoration: underline;
    font-size: 0.78rem;
    cursor: pointer;
  }

  .spinner {
    width: 26px;
    height: 26px;
    border: 3px solid var(--color-surface);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .mini-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
  }

  .spin {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .restart-modal-content {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .modal-header-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .modal-icon-wrap.warning {
    color: var(--color-warning);
    display: flex;
    align-items: center;
    background-color: color-mix(in srgb, var(--color-warning) 20%, var(--color-surface));
    padding: 0.5rem;
    border-radius: 0.4rem;
  }

  .modal-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .modal-subtitle {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .modal-body-text {
    font-size: 0.85rem;
    color: var(--color-text);
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .modal-body-text p {
    margin: 0;
  }

  .modal-note {
    font-size: 0.78rem;
    color: var(--color-text-muted);
  }

  .modal-footer-actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 0.6rem;
    margin-top: 0.5rem;
  }

  .btn-modal-cancel {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.4rem 0.9rem;
    border-radius: 0.3rem;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-modal-cancel:hover {
    background-color: var(--color-surface-hover);
  }

  .btn-modal-confirm-restart {
    background-color: var(--color-warning);
    border: solid 1px var(--color-warning);
    color: var(--color-base);
    padding: 0.4rem 1rem;
    border-radius: 0.3rem;
    font-size: 0.82rem;
    font-weight: 700;
    cursor: pointer;
  }

  .btn-modal-confirm-restart:hover {
    filter: brightness(1.15);
  }
</style>
