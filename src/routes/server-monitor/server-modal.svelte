<script lang="ts">
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

  let activeTab = $state<"details" | "history">("details");
  let label = $state("");
  let host = $state("");
  let description = $state("");
  let hasAgent = $state(false);
  let agentPort = $state(9001);
  let agentApiKey = $state("");
  let confirmDelete = $state(false);

  let logs = $state<ServerLog[]>([]);
  let logsLoading = $state(false);
  let logsError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      confirmDelete = false;
      activeTab = "details";

      if (server) {
        label = server.label;
        host = server.host;
        description = server.description ?? "";
        hasAgent = !!server.agent;
        agentPort = server.agent?.port ?? 9001;
        agentApiKey = server.agent?.api_key ?? "";
        fetchLogs(server.id);
      } else {
        label = "";
        host = "";
        description = "";
        hasAgent = false;
        agentPort = 9001;
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
        limit: 50,
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
    if (!confirmDelete) {
      confirmDelete = true;
      return;
    }
    if (server) {
      await ondelete(server.id);
    }
    open = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleBackdropKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
    }
  }

  function formatTimestamp(isoString: string): string {
    const date = new Date(isoString);
    if (isNaN(date.getTime())) return isoString;
    return date.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
</script>

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeyDown}
    role="presentation"
  >
    <div
      class="modal-card"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
    >
      <div class="modal-header">
        <div class="header-main">
          <h2 class="modal-title">{server ? server.label : "Add Server"}</h2>
          {#if server}
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
        </div>

        <div class="modal-footer">
          {#if server}
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
        <div class="history-body">
          {#if logsLoading}
            <p class="history-state">Loading history...</p>
          {:else if logsError}
            <p class="history-state error">{logsError}</p>
          {:else if logs.length === 0}
            <p class="history-state">No connection events recorded yet.</p>
          {:else}
            <div class="log-list">
              {#each logs as log (log.id)}
                <div class="log-item">
                  <span
                    class="log-badge"
                    class:connected={log.event_type === "connected"}
                    class:disconnected={log.event_type === "disconnected"}
                  >
                    {log.event_type}
                  </span>
                  <div class="log-details">
                    <span class="log-time">{formatTimestamp(log.timestamp)}</span>
                    {#if log.latency_ms !== undefined && log.latency_ms !== null}
                      <span class="log-latency">{log.latency_ms} ms</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <div class="modal-footer">
          <div class="footer-actions">
            <button type="button" class="btn-cancel" onclick={() => (open = false)}>
              Close
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-card {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    width: 100%;
    max-width: 30rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    outline: none;
    max-height: 85vh;
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .header-main {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .modal-title {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .tab-nav {
    display: flex;
    gap: 0.5rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .tab-btn {
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 0.35rem 0.5rem;
    cursor: pointer;
    margin-bottom: -1px;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--color-text);
  }

  .tab-btn.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .history-body {
    overflow-y: auto;
    max-height: 20rem;
    padding-right: 0.25rem;
  }

  .history-state {
    margin: 1.5rem 0;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }

  .history-state.error {
    color: var(--color-failure);
  }

  .log-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .log-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0.75rem;
    border-radius: 0.375rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    font-size: 0.85rem;
  }

  .log-badge {
    text-transform: capitalize;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    border: solid var(--width-border) currentColor;
  }

  .log-badge.connected {
    color: var(--color-primary);
    background-color: color-mix(in srgb, var(--color-primary) 15%, transparent);
  }

  .log-badge.disconnected {
    color: var(--color-failure);
    background-color: color-mix(in srgb, var(--color-failure) 15%, transparent);
  }

  .log-details {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .log-latency {
    font-family: monospace;
    font-variant-numeric: tabular-nums;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .form-field input {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.65rem;
    color: var(--color-text);
    font-size: 0.9rem;
    outline: none;
  }

  .form-field input:focus {
    border-color: var(--color-primary);
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

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.5rem;
  }

  .footer-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .btn-delete {
    background-color: transparent;
    border: solid var(--width-border) var(--color-failure);
    color: var(--color-failure);
    padding: 0.45rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-delete.confirming {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .btn-cancel {
    background-color: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.45rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .btn-save {
    background-color: var(--color-primary);
    border: solid var(--width-border) var(--color-primary);
    color: var(--color-base);
    padding: 0.45rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
