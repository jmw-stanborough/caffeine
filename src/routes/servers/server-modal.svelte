<script lang="ts">
  import ConfigHistoryModal from "$lib/components/config-history-modal.svelte";
  import { DEFAULT_AGENT_PORT, DEFAULT_LOG_LIMIT } from "$lib/constants";
  import { invoke } from "@tauri-apps/api/core";
  import type { Server, ServerLog } from "./types";

  let {
    open = $bindable(false),
    server = undefined,
    onsave,
    ondelete,
  }: {
    open: boolean;
    server?: Server;
    onsave: (saved: Server, existingId?: string) => Promise<void> | void;
    ondelete: (id: string) => Promise<void> | void;
  } = $props();

  let label = $state("");
  let host = $state("");
  let description = $state("");
  let hasAgent = $state(false);
  let agentPort = $state(DEFAULT_AGENT_PORT);
  let agentApiKey = $state("");

  let logs = $state<ServerLog[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      if (server) {
        label = server.label;
        host = server.host;
        description = server.description ?? "";
        hasAgent = Boolean(server.agent);
        agentPort = server.agent?.port ?? DEFAULT_AGENT_PORT;
        agentApiKey = server.agent?.api_key ?? "";
        fetchLogs(server.id);
      } else {
        label = "";
        host = "";
        description = "";
        hasAgent = false;
        agentPort = DEFAULT_AGENT_PORT;
        agentApiKey = "";
        logs = [];
      }
    }
  });

  async function fetchLogs(serverId: string) {
    logsLoading = true;
    logsError = null;
    try {
      logs = await invoke<ServerLog[]>("get_server_logs", {
        serverId,
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

    const payload: Server = {
      id: server?.id ?? "",
      host: host.trim(),
      label: label.trim(),
      description: description.trim() || undefined,
      agent: hasAgent
        ? {
            port: Number(agentPort),
            api_key: agentApiKey.trim(),
          }
        : undefined,
    };

    await onsave(payload, server?.id);
    open = false;
  }

  async function handleDelete() {
    if (server) {
      await ondelete(server.id);
    }
  }
</script>

<ConfigHistoryModal
  bind:open
  title={server ? server.label : "Add Server"}
  isExisting={Boolean(server)}
  maxWidth="30rem"
  {logs}
  {logsLoading}
  {logsError}
  onsave={handleSave}
  ondelete={handleDelete}
>
  <label class="form-field">
    <span>Label</span>
    <input type="text" bind:value={label} placeholder="e.g. Production Web" />
  </label>

  <label class="form-field">
    <span>Host / IP</span>
    <input type="text" bind:value={host} placeholder="e.g. 192.168.1.10" />
  </label>

  <label class="form-field">
    <span>Description (Optional)</span>
    <input type="text" bind:value={description} placeholder="e.g. Primary web gateway" />
  </label>

  <label class="checkbox-field">
    <input type="checkbox" bind:checked={hasAgent} />
    <span>Enable Agent Integration</span>
  </label>

  {#if hasAgent}
    <div class="agent-fields">
      <label class="form-field">
        <span>Agent Port</span>
        <input type="number" bind:value={agentPort} />
      </label>
      <label class="form-field">
        <span>Agent API Key</span>
        <input type="password" bind:value={agentApiKey} />
      </label>
    </div>
  {/if}
</ConfigHistoryModal>

<style>
  .checkbox-field {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--color-text);
    cursor: pointer;
    margin-top: 0.25rem;
  }

  .agent-fields {
    display: flex;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: color-mix(in srgb, var(--color-base) 50%, transparent);
    border-radius: 0.375rem;
  }
</style>
