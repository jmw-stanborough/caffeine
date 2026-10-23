<script lang="ts">
  import EventHistoryList from "$lib/components/event-history-list.svelte";
  import Modal from "$lib/components/modal.svelte";
  import type { Snippet } from "svelte";

  interface LogItem {
    id: number;
    event_type: string;
    latency_ms?: number;
    timestamp: string;
  }

  let {
    open = $bindable(false),
    title,
    isExisting = false,
    maxWidth = "32rem",
    logs = [],
    logsLoading = false,
    logsError = null,
    onsave,
    ondelete,
    children,
  }: {
    open: boolean;
    title: string;
    isExisting?: boolean;
    maxWidth?: string;
    logs?: LogItem[];
    logsLoading?: boolean;
    logsError?: string | null;
    onsave: () => Promise<void> | void;
    ondelete: () => Promise<void> | void;
    children: Snippet;
  } = $props();

  let activeTab = $state<"details" | "history">("details");
  let confirmDelete = $state(false);

  $effect(() => {
    if (open) {
      confirmDelete = false;
      activeTab = "details";
    }
  });

  async function handleDelete() {
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    await ondelete();
    open = false;
  }
</script>

<Modal bind:open {maxWidth} ariaLabel={title}>
  <div class="modal-header">
    <div class="header-main">
      <h2 class="modal-title">{title}</h2>
      {#if isExisting}
        <div class="tab-nav">
          <button
            type="button"
            class="tab-btn"
            class:active={activeTab === "details"}
            onclick={() => (activeTab = "details")}
          >
            Configuration
          </button>
          <button
            type="button"
            class="tab-btn"
            class:active={activeTab === "history"}
            onclick={() => (activeTab = "history")}
          >
            Event History
          </button>
        </div>
      {/if}
    </div>
    <button type="button" class="btn-close" onclick={() => (open = false)}>✕</button>
  </div>

  {#if activeTab === "details"}
    <div class="modal-body">
      {@render children()}
    </div>

    <div class="modal-footer">
      {#if isExisting}
        <button
          type="button"
          class="btn-delete"
          class:confirming={confirmDelete}
          onclick={handleDelete}
        >
          {confirmDelete ? "Click to Confirm Delete" : "Delete"}
        </button>
      {/if}

      <div class="footer-actions">
        <button type="button" class="btn-cancel" onclick={() => (open = false)}>
          Cancel
        </button>
        <button type="button" class="btn-save" onclick={onsave}>
          Save
        </button>
      </div>
    </div>
  {:else}
    <EventHistoryList
      {logs}
      loading={logsLoading}
      error={logsError}
      onclose={() => (open = false)}
    />
  {/if}
</Modal>
