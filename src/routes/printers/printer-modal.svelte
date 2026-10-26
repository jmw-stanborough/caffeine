<script lang="ts">
  import ConfigHistoryModal from "$lib/components/config-history-modal.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import { DEFAULT_LOG_LIMIT, DEFAULT_SNMP_COMMUNITY } from "$lib/constants";
  import { invoke } from "@tauri-apps/api/core";
  import type { Printer, PrinterLog } from "./types";

  let {
    open = $bindable(false),
    printer = undefined,
    readOnly,
    onsave,
    ondelete,
  }: {
    open: boolean;
    printer?: Printer;
    readOnly?: boolean;
    onsave: (saved: Printer, existingId?: string) => Promise<void> | void;
    ondelete: (id: string) => Promise<void> | void;
  } = $props();

  let isReadOnly = $derived(readOnly ?? !clientMode.canEdit);

  let label = $state("");
  let host = $state("");
  let description = $state("");
  let snmpCommunity = $state(DEFAULT_SNMP_COMMUNITY);

  let logs = $state<PrinterLog[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      if (printer) {
        label = printer.label;
        host = printer.host;
        description = printer.description ?? "";
        snmpCommunity = printer.snmp_community || DEFAULT_SNMP_COMMUNITY;
        fetchLogs(printer.id);
      } else {
        label = "";
        host = "";
        description = "";
        snmpCommunity = DEFAULT_SNMP_COMMUNITY;
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
    if (isReadOnly || !host.trim() || !label.trim()) return;

    const payload: Printer = {
      id: printer?.id ?? "",
      host: host.trim(),
      label: label.trim(),
      description: description.trim() || undefined,
      model: printer?.model,
      location: printer?.location,
      snmp_community: snmpCommunity.trim() || DEFAULT_SNMP_COMMUNITY,
      web_url: printer?.web_url,
    };

    await onsave(payload, printer?.id);
    open = false;
  }

  async function handleDelete() {
    if (isReadOnly || !printer) return;
    await ondelete(printer.id);
  }
</script>

<ConfigHistoryModal
  bind:open
  title={printer ? printer.label : "Add Printer"}
  isExisting={Boolean(printer)}
  readOnly={isReadOnly}
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

  <label class="form-field">
    <span>Description / Notes (Optional)</span>
    <input type="text" bind:value={description} placeholder="e.g. Main duplex color printer" />
  </label>

  <label class="form-field">
    <span>SNMP v2c Community</span>
    <input type="text" bind:value={snmpCommunity} placeholder={DEFAULT_SNMP_COMMUNITY} />
  </label>
</ConfigHistoryModal>

<style>
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
</style>
