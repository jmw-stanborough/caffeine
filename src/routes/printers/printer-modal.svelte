<script lang="ts">
  import EventHistoryList from "$lib/components/event-history-list.svelte";
  import Modal from "$lib/components/modal.svelte";
  import { DEFAULT_LOG_LIMIT, DEFAULT_SNMP_COMMUNITY } from "$lib/constants";
  import { invoke } from "@tauri-apps/api/core";
  import type { Printer, PrinterLog } from "./types";

  let {
    open = $bindable(false),
    printer = undefined,
    onsave,
    ondelete,
  }: {
    open: boolean;
    printer?: Printer;
    onsave: (saved: Printer, existingId?: string) => Promise<void> | void;
    ondelete: (id: string) => Promise<void> | void;
  } = $props();

  let activeTab = $state<"details" | "history">("details");
  let label = $state("");
  let host = $state("");
  let description = $state("");
  let model = $state("");
  let location = $state("");
  let snmpCommunity = $state(DEFAULT_SNMP_COMMUNITY);
  let webUrl = $state("");
  let confirmDelete = $state(false);

  let logs = $state<PrinterLog[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      confirmDelete = false;
      activeTab = "details";

      if (printer) {
        label = printer.label;
        host = printer.host;
        description = printer.description ?? "";
        model = printer.model ?? "";
        location = printer.location ?? "";
        snmpCommunity = printer.snmp_community || DEFAULT_SNMP_COMMUNITY;
        webUrl = printer.web_url ?? "";
        fetchLogs(printer.id);
      } else {
        label = "";
        host = "";
        description = "";
        model = "";
        location = "";
        snmpCommunity = DEFAULT_SNMP_COMMUNITY;
        webUrl = "";
        logs = [];
      }
    }
  });

  async function fetchLogs(printerId: string) {
    logsLoading = true;
    logsError = null;
    try {
      logs = await invoke<PrinterLog[]>("get_printer_logs", {
        printerId,
        limit: DEFAULT_LOG_LIMIT,
      });
    } catch (err) {
      logsError = String(err);
    } finally {
      logsLoading = false;
    }
  }

  async function handleSave() {
    if (!host.trim() || !label.trim()) return;

    const payload: Printer = {
      id: printer?.id ?? "",
      host: host.trim(),
      label: label.trim(),
      description: description.trim() || undefined,
      model: model.trim() || undefined,
      location: location.trim() || undefined,
      snmp_community: snmpCommunity.trim() || DEFAULT_SNMP_COMMUNITY,
      web_url: webUrl.trim() || undefined,
    };

    await onsave(payload, printer?.id);
    open = false;
  }

  async function handleDelete() {
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    if (printer) {
      await ondelete(printer.id);
    }
    open = false;
  }
</script>

<Modal bind:open maxWidth="36rem" ariaLabel={printer ? printer.label : "Add Printer"}>
  <div class="modal-header">
    <div class="header-main">
      <h2 class="modal-title">{printer ? printer.label : "Add Printer"}</h2>
      {#if printer}
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
      <div class="field-row">
        <label class="form-field">
          <span>Label</span>
          <input type="text" bind:value={label} placeholder="e.g. 2nd Floor Reception" />
        </label>

        <label class="form-field">
          <span>Host / IP</span>
          <input type="text" bind:value={host} placeholder="e.g. 192.168.1.50" />
        </label>
      </div>

      <div class="field-row">
        <label class="form-field">
          <span>Model (Optional — auto-detected via SNMP)</span>
          <input type="text" bind:value={model} placeholder="e.g. HP LaserJet Enterprise M611" />
        </label>

        <label class="form-field">
          <span>Location (Optional)</span>
          <input type="text" bind:value={location} placeholder="e.g. Room 204 / Copy Room" />
        </label>
      </div>

      <label class="form-field">
        <span>Description / Notes (Optional)</span>
        <input type="text" bind:value={description} placeholder="e.g. Main duplex color printer" />
      </label>

      <div class="field-row">
        <label class="form-field">
          <span>Web Interface URL (Optional)</span>
          <input type="text" bind:value={webUrl} placeholder="Defaults to http://<host>" />
        </label>

        <label class="form-field">
          <span>SNMP v2c Community</span>
          <input type="text" bind:value={snmpCommunity} placeholder={DEFAULT_SNMP_COMMUNITY} />
        </label>
      </div>
    </div>

    <div class="modal-footer">
      {#if printer}
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
        <button type="button" class="btn-save" onclick={handleSave}>
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

<style>
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
</style>
