<script lang="ts">
  interface Agent {
    port: number;
    api_key: string;
  }

  interface Server {
    host: string;
    label: string;
    description?: string;
    agent?: Agent;
  }

  let {
    open = $bindable(false),
    server = undefined,
    onsave,
    ondelete
  }: {
    open: boolean;
    server?: Server;
    onsave: (savedServer: Server, originalHost?: string) => void;
    ondelete: (host: string) => void;
  } = $props();

  let host = $state("");
  let label = $state("");
  let description = $state("");
  let hasAgent = $state(false);
  let agentPort = $state<number>(8080);
  let agentApiKey = $state("");
  let confirmingDelete = $state(false);

  $effect(() => {
    if (server) {
      host = server.host;
      label = server.label;
      description = server.description ?? "";
      hasAgent = !!server.agent;
      agentPort = server.agent?.port ?? 8080;
      agentApiKey = server.agent?.api_key ?? "";
    } else {
      host = "";
      label = "";
      description = "";
      hasAgent = false;
      agentPort = 8080;
      agentApiKey = "";
    }
    confirmingDelete = false;
  });

  function close() {
    open = false;
    confirmingDelete = false;
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!host.trim() || !label.trim()) return;

    const payload: Server = {
      host: host.trim(),
      label: label.trim(),
      description: description.trim() || undefined,
      agent: hasAgent
        ? {
            port: Number(agentPort),
            api_key: agentApiKey.trim()
          }
        : undefined
    };

    onsave(payload, server?.host);
    close();
  }

  function handleDelete() {
    if (!server) return;
    if (!confirmingDelete) {
      confirmingDelete = true;
      return;
    }
    ondelete(server.host);
    close();
  }
</script>

{#if open}
  <div class="modal-backdrop" onclick={close} role="presentation">
    <div
      class="modal-card"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
    >
      <header class="modal-header">
        <h2 id="modal-title">{server ? "Edit Server" : "Add Server"}</h2>
        <button type="button" class="btn-close" onclick={close} aria-label="Close modal">✕</button>
      </header>

      <form onsubmit={handleSubmit} class="modal-form">
        <div class="field">
          <label for="server-label">Label</label>
          <input
            id="server-label"
            type="text"
            required
            placeholder="e.g. Primary DNS"
            bind:value={label}
          />
        </div>

        <div class="field">
          <label for="server-host">Host / IP</label>
          <input
            id="server-host"
            type="text"
            required
            placeholder="e.g. 172.19.20.245"
            bind:value={host}
          />
        </div>

        <div class="field">
          <label for="server-desc">Description (Optional)</label>
          <input
            id="server-desc"
            type="text"
            placeholder="e.g. Secondary DHCP/DNS server"
            bind:value={description}
          />
        </div>

        <div class="checkbox-row">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={hasAgent} />
            <span>Enable Agent Integration</span>
          </label>
        </div>

        {#if hasAgent}
          <div class="agent-fields">
            <div class="field">
              <label for="agent-port">Agent Port</label>
              <input id="agent-port" type="number" min="1" max="65535" bind:value={agentPort} />
            </div>

            <div class="field">
              <label for="agent-key">API Key</label>
              <input id="agent-key" type="password" bind:value={agentApiKey} />
            </div>
          </div>
        {/if}

        <footer class="modal-actions">
          {#if server}
            <button
              type="button"
              class="btn-delete"
              class:confirming={confirmingDelete}
              onclick={handleDelete}
            >
              {confirmingDelete ? "Click to Confirm Delete" : "Delete"}
            </button>
          {/if}

          <div class="actions-right">
            <button type="button" class="btn-cancel" onclick={close}>Cancel</button>
            <button type="submit" class="btn-submit">Save</button>
          </div>
        </footer>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    width: 100%;
    max-width: 28rem;
    padding: 1.5rem;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--color-text);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
  }

  .modal-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field label {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .field input {
    background-color: color-mix(in srgb, var(--color-surface) 70%, black);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.75rem;
    color: var(--color-text);
    outline: none;
  }

  .field input:focus {
    border-color: var(--color-primary);
  }

  .checkbox-row {
    margin-top: 0.25rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .agent-fields {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.75rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
  }

  .modal-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.5rem;
    padding-top: 1rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .actions-right {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .btn-delete {
    background-color: transparent;
    color: #ef4444;
    border: solid var(--width-border) #ef4444;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .btn-delete:hover,
  .btn-delete.confirming {
    background-color: #ef4444;
    color: #ffffff;
  }

  .btn-cancel {
    background: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .btn-submit {
    background-color: var(--color-primary);
    color: var(--color-background);
    border: none;
    font-weight: 600;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    cursor: pointer;
  }
</style>
