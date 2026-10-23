<script lang="ts">
  import ConfigHistoryModal from "$lib/components/config-history-modal.svelte";
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

  let label = $state("");
  let host = $state("");
  let description = $state("");
  let model = $state("");
  let location = $state("");
  let snmpCommunity = $state(DEFAULT_SNMP_COMMUNITY);
  let webUrl = $state("");

  let logs = $state<PrinterLog[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
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
    if (printer) {
      await ondelete(printer.id);
    }
  }
</script>

<ConfigHistoryModal
  bind:open
  title={printer ? printer.label : "Add Printer"}
  isExisting={Boolean(printer)}
  maxWidth="36rem"
  {logs}
  {logsLoading}
  {logsError}
  onsave={handleSave}
  ondelete={handleDelete}
>
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
</ConfigHistoryModal>

<style>
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
</style>
