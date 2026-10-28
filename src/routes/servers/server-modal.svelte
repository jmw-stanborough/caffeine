<script lang="ts">
  import ConfigHistoryModal from "$lib/components/config-history-modal.svelte";
  import { clientMode } from "$lib/client-mode.svelte";
  import { DEFAULT_AGENT_PORT, DEFAULT_LOG_LIMIT } from "$lib/constants";
  import { invoke } from "@tauri-apps/api/core";
  import type { Server, ServerLog } from "./types";

  let {
    open = $bindable(false),
    server = undefined,
    readOnly,
    onsave,
    ondelete,
  }: {
    open: boolean;
    server?: Server;
    readOnly?: boolean;
    onsave: (saved: Server, existingId?: string) => Promise<void> | void;
    ondelete: (id: string) => Promise<void> | void;
  } = $props();

  let isReadOnly = $derived(readOnly ?? !clientMode.canEdit);

  let label = $state("");
  let host = $state("");
  let description = $state("");
  let hasAgent = $state(false);
  let agentPort = $state(DEFAULT_AGENT_PORT);
  let agentApiKey = $state("");
  let sshPort = $state<number | undefined>(22);
  let rdpPort = $state<number | undefined>(3389);
  let remoteUsername = $state("");

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
        sshPort = server.ssh_port ?? 22;
        rdpPort = server.rdp_port ?? 3389;
        remoteUsername = server.remote_username ?? "";
        fetchLogs(server.id);
      } else {
        label = "";
        host = "";
        description = "";
        hasAgent = false;
        agentPort = DEFAULT_AGENT_PORT;
        agentApiKey = "";
        sshPort = 22;
        rdpPort = 3389;
        remoteUsername = "";
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
    if (isReadOnly || !host.trim() || !label.trim()) return;

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
      ssh_port: sshPort ? Number(sshPort) : undefined,
      rdp_port: rdpPort ? Number(rdpPort) : undefined,
      remote_username: remoteUsername.trim() || undefined,
    };

    await onsave(payload, server?.id);
    open = false;
  }

  async function handleDelete() {
    if (isReadOnly || !server) return;
    await ondelete(server.id);
  }
</script>

<ConfigHistoryModal
  bind:open
  title={server ? server.label : "Add Server"}
  isExisting={Boolean(server)}
  readOnly={isReadOnly}
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

  <div class="remote-fields-group">
    <div class="remote-fields-header">Remote Access (Optional)</div>
    <label class="form-field">
      <span>Remote Username</span>
      <input type="text" bind:value={remoteUsername} placeholder="e.g. administrator or root" />
    </label>
    <div class="ports-row">
      <label class="form-field">
        <span>SSH Port</span>
        <input type="number" bind:value={sshPort} placeholder="22" />
      </label>
      <label class="form-field">
        <span>RDP Port</span>
        <input type="number" bind:value={rdpPort} placeholder="3389" />
      </label>
    </div>
  </div>

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
  .remote-fields-group {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 0.75rem;
    background-color: color-mix(in srgb, var(--color-base) 40%, transparent);
    border: solid 1px var(--color-border);
    border-radius: 0.375rem;
    margin-top: 0.25rem;
  }

  .remote-fields-header {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .ports-row {
    display: flex;
    gap: 0.75rem;
  }

  .ports-row .form-field {
    flex: 1;
  }

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
