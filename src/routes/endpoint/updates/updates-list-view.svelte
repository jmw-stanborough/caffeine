<script lang="ts">
  import type { EndpointStore } from "../endpoint.svelte";
  import type { SystemUpdateItem } from "../types";
  import UpdateCard from "../update-card.svelte";

  interface Props {
    updates: SystemUpdateItem[];
    selectedIds: Set<string>;
    allFilteredSelected: boolean;
    store: EndpointStore;
    isBusy: boolean;
    isRebootRequired: boolean;
    onToggleSelectAll: () => void;
    onToggleSelectOne: (id: string) => void;
    onDownload: (target?: string | string[]) => void;
    onInstall: (target?: string | string[]) => void;
    onDownloadAndInstall: (target?: string | string[]) => void;
    onCancelQueue: (id: string) => void;
  }

  let {
    updates,
    selectedIds,
    allFilteredSelected,
    store,
    isBusy,
    isRebootRequired,
    onToggleSelectAll,
    onToggleSelectOne,
    onDownload,
    onInstall,
    onDownloadAndInstall,
    onCancelQueue,
  }: Props = $props();
</script>

<div class="updates-list-header">
  <label class="select-all-label">
    <input
      type="checkbox"
      checked={allFilteredSelected}
      onchange={onToggleSelectAll}
      class="card-checkbox"
    />
    <span>Select all shown ({updates.length})</span>
  </label>
</div>

<div class="updates-list">
  {#each updates as update (update.id)}
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
      onToggleSelect={onToggleSelectOne}
      {onDownload}
      {onInstall}
      {onDownloadAndInstall}
      {onCancelQueue}
    />
  {/each}
</div>

<style>
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
</style>
