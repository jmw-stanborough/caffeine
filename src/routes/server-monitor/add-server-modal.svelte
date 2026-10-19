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
    onsave
  }: {
    open: boolean;
    onsave: (server: Server) => Promise<void> | void;
  } = $props();

  let formLabel = $state("");
  let formHost = $state("");
  let formDescription = $state("");
  let formHasAgent = $state(false);
  let formAgentPort = $state(9100);
  let formAgentKey = $state("");
  let isSubmitting = $state(false);

  function resetForm() {
    formLabel = "";
    formHost = "";
    formDescription = "";
    formHasAgent = false;
    formAgentPort = 9100;
    formAgentKey = "";
  }

  function handleClose() {
    resetForm();
    open = false;
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!formLabel.trim() || !formHost.trim()) return;

    isSubmitting = true;
    try {
      await onsave({
        label: formLabel.trim(),
        host: formHost.trim(),
        description: formDescription.trim() || undefined,
        agent: formHasAgent
          ? {
              port: Number(formAgentPort),
              api_key: formAgentKey.trim()
            }
          : undefined
      });
      handleClose();
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if open}
  <div class="modal-backdrop" onclick={handleClose} role="presentation">
    <div
      class="modal-content"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-heading"
    >
      <h2 id="modal-heading">Add New Server</h2>

      <form class="modal-form" onsubmit={handleSubmit}>
        <label>
          <span>Label *</span>
          <input type="text" bind:value={formLabel} placeholder="VSVR-001" required />
        </label>

        <label>
          <span>Host / IP *</span>
          <input type="text" bind:value={formHost} placeholder="172.19.20.142" required />
        </label>

        <label>
          <span>Description</span>
          <input type="text" bind:value={formDescription} placeholder="Primary DHCP/DNS server" />
        </label>

        <fieldset class="agent-fieldset">
          <label class="checkbox-row">
            <input type="checkbox" bind:checked={formHasAgent} />
            <span>Enable Monitoring Agent</span>
          </label>

          {#if formHasAgent}
            <div class="agent-inputs">
              <label>
                <span>Agent Port</span>
                <input type="number" bind:value={formAgentPort} />
              </label>
              <label>
                <span>API Key</span>
                <input type="password" bind:value={formAgentKey} />
              </label>
            </div>
          {/if}
        </fieldset>

        <div class="modal-actions">
          <button type="button" class="btn-cancel" onclick={handleClose}>Cancel</button>
          <button type="submit" class="btn-submit" disabled={isSubmitting}>
            {isSubmitting ? "Saving..." : "Save Server"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-content {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    padding: 1.5rem;
    width: 100%;
    max-width: 24rem;
  }

  .modal-content h2 {
    margin: 0 0 1rem 0;
    color: var(--color-primary);
    font-size: 1.25rem;
  }

  .modal-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .modal-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .modal-form input[type="text"],
  .modal-form input[type="number"],
  .modal-form input[type="password"] {
    padding: 0.5rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
  }

  .agent-fieldset {
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.75rem;
    margin: 0.25rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .checkbox-row {
    flex-direction: row !important;
    align-items: center;
    gap: 0.5rem;
  }

  .agent-inputs {
    display: flex;
    gap: 0.5rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .btn-cancel,
  .btn-submit {
    padding: 0.5rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .btn-cancel {
    background: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-submit {
    background-color: var(--color-primary);
    border: none;
    color: var(--color-base);
    font-weight: 600;
  }

  .btn-submit:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
