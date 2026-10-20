<script lang="ts">
  import type { Server } from "./types";

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
  let agentPort = $state(9001);
  let agentApiKey = $state("");
  let confirmDelete = $state(false);

  $effect(() => {
    if (open) {
      confirmDelete = false;
      if (server) {
        label = server.label;
        host = server.host;
        description = server.description ?? "";
        hasAgent = !!server.agent;
        agentPort = server.agent?.port ?? 9001;
        agentApiKey = server.agent?.api_key ?? "";
      } else {
        label = "";
        host = "";
        description = "";
        hasAgent = false;
        agentPort = 9001;
        agentApiKey = "";
      }
    }
  });

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
</script>

{#if open}
  <div class="modal-backdrop" onclick={() => (open = false)} role="presentation">
    <div class="modal-card" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
      <div class="modal-header">
        <h2 class="modal-title">{server ? "Edit Server" : "Add Server"}</h2>
        <button type="button" class="btn-close" onclick={() => (open = false)}>✕</button>
      </div>

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
    max-width: 28rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-title {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
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
    border: solid var(--width-border) var(--color-error);
    color: var(--color-error);
    padding: 0.45rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-delete.confirming {
    background-color: var(--color-error);
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
