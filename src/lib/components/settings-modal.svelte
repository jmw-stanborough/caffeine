<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import { Circle } from "$lib/components/icons";
  import { applyTheme } from "$lib/theme";
  import { refreshClientMode } from "$lib/client-mode.svelte";
  import { printerStore } from "../../routes/printers/printers.svelte";
  import { serverStore } from "../../routes/servers/servers.svelte";
  import { invoke } from "@tauri-apps/api/core";

  export interface ServerTokenPublic {
    id: string;
    label: string;
    created_at: string;
    token_preview: string;
  }

  export interface GeneratedTokenResponse {
    id: string;
    label: string;
    token: string;
    created_at: string;
  }

  export interface AppSettings {
    target_host: string;
    target_port: number;
    server_mode: boolean;
    server_port: number;
    server_bind: string;
    server_tokens?: ServerTokenPublic[];
    server_admin_token?: string | null;
    admin_token: string | null;
    theme: string;
    retention_days: number;
    max_log_entries: number;
  }

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();

  let targetHost = $state("127.0.0.1");
  let targetPort = $state(6310);
  let serverMode = $state(false);
  let serverPort = $state(6310);
  let serverBind = $state("0.0.0.0");
  let serverTokens = $state<ServerTokenPublic[]>([]);
  let newLabel = $state("");
  let generatingToken = $state(false);
  let tokenActionError = $state<string | null>(null);
  let revealedSecret = $state<GeneratedTokenResponse | null>(null);
  let secretCopied = $state(false);
  let adminToken = $state("");
  let theme = $state("system");
  let persistedTheme = $state("system");
  let retentionDays = $state(14);
  let maxLogEntries = $state(1000);

  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let successMsg = $state<string | null>(null);

  let isRemoteClient = $derived(
    targetHost.trim() !== "127.0.0.1" &&
      targetHost.trim().toLowerCase() !== "localhost" &&
      targetHost.trim() !== "::1" &&
      targetHost.trim() !== "0.0.0.0"
  );

  $effect(() => {
    if (open) {
      loadSettings();
    }
  });

  function handleClose() {
    theme = persistedTheme;
    applyTheme(persistedTheme);
    open = false;
  }

  async function loadServerTokens() {
    try {
      serverTokens = await invoke<ServerTokenPublic[]>("get_server_tokens_cmd");
    } catch (err) {
      tokenActionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleCreateToken() {
    if (!newLabel.trim()) return;
    generatingToken = true;
    tokenActionError = null;
    try {
      const generated = await invoke<GeneratedTokenResponse>("create_server_token_cmd", {
        label: newLabel.trim(),
      });
      revealedSecret = generated;
      newLabel = "";
      await loadServerTokens();
    } catch (err) {
      tokenActionError = err instanceof Error ? err.message : String(err);
    } finally {
      generatingToken = false;
    }
  }

  async function handleRegenerateToken(token: ServerTokenPublic) {
    if (
      !confirm(
        `Regenerating the token for "${token.label}" will immediately revoke the previous key. Any remote clients using it will lose access until updated. Proceed?`
      )
    ) {
      return;
    }
    tokenActionError = null;
    try {
      const regenerated = await invoke<GeneratedTokenResponse>(
        "regenerate_server_token_cmd",
        { id: token.id }
      );
      revealedSecret = regenerated;
      await loadServerTokens();
    } catch (err) {
      tokenActionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function handleDeleteToken(token: ServerTokenPublic) {
    if (
      !confirm(
        `Are you sure you want to revoke the token for "${token.label}"? Any remote clients using it will immediately be rejected.`
      )
    ) {
      return;
    }
    tokenActionError = null;
    try {
      await invoke("delete_server_token_cmd", { id: token.id });
      await loadServerTokens();
    } catch (err) {
      tokenActionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function copyRevealedSecret() {
    if (!revealedSecret) return;
    try {
      await navigator.clipboard.writeText(revealedSecret.token);
      secretCopied = true;
      setTimeout(() => {
        secretCopied = false;
      }, 2500);
    } catch (e) {
      console.warn("Failed to copy token", e);
    }
  }

  async function loadSettings() {
    loading = true;
    error = null;
    successMsg = null;
    try {
      const settings = await invoke<AppSettings>("get_app_settings_cmd");
      targetHost = settings.target_host;
      targetPort = settings.target_port;
      serverMode = settings.server_mode;
      serverPort = settings.server_port;
      serverBind = settings.server_bind;
      serverTokens = settings.server_tokens ?? [];
      adminToken = settings.admin_token ?? "";
      persistedTheme = settings.theme;
      theme = settings.theme;
      applyTheme(persistedTheme);
      retentionDays = settings.retention_days;
      maxLogEntries = settings.max_log_entries;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    saving = true;
    error = null;
    successMsg = null;
    try {
      const payload: AppSettings = {
        target_host: targetHost.trim() || "127.0.0.1",
        target_port: Number(targetPort) || 6310,
        server_mode: serverMode,
        server_port: Number(serverPort) || 6310,
        server_bind: serverBind.trim() || "0.0.0.0",
        admin_token: adminToken.trim() ? adminToken.trim() : null,
        theme,
        retention_days: Number(retentionDays) || 14,
        max_log_entries: Number(maxLogEntries) || 1000,
      };

      await invoke<AppSettings>("update_app_settings_cmd", {
        settings: payload,
      });

      await Promise.allSettled([
        serverStore.fetchServers(),
        printerStore.fetchPrinters(),
        refreshClientMode(),
      ]);

      persistedTheme = theme;
      applyTheme(theme);
      successMsg = "Settings saved successfully";
      setTimeout(() => {
        open = false;
      }, 600);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal bind:open maxWidth="38rem" ariaLabel="Application Settings" onBackdropClose={handleClose}>
  <div class="settings-header">
    <h2>Application Settings</h2>
    <button
      type="button"
      class="close-button"
      onclick={handleClose}
      aria-label="Close settings"
    >
      &times;
    </button>
  </div>

  {#if loading}
    <div class="settings-loading">Loading settings...</div>
  {:else}
    <form
      class="settings-form"
      onsubmit={(e) => {
        e.preventDefault();
        handleSave();
      }}
    >
      {#if error}
        <div class="alert alert-error">{error}</div>
      {/if}
      {#if successMsg}
        <div class="alert alert-success">{successMsg}</div>
      {/if}

      <section class="settings-section">
        <h3 class="section-title">Connection Target</h3>
        <p class="section-description">
          Defaults to <code>127.0.0.1</code> (self-serving client). Point to an IP or hostname to consume from a remote Caffeine instance.
        </p>

        <div class="form-row">
          <div class="form-group flex-2">
            <label for="targetHost">Target Host / IP</label>
            <input
              id="targetHost"
              type="text"
              bind:value={targetHost}
              placeholder="127.0.0.1 or server.domain.lan"
              required
            />
          </div>
          <div class="form-group flex-1">
            <label for="targetPort">Target Port</label>
            <input
              id="targetPort"
              type="number"
              bind:value={targetPort}
              min="1"
              max="65535"
              required
            />
          </div>
        </div>

        {#if isRemoteClient}
          <div class="remote-badge">
            <Circle size={8} class="status-indicator" />
            Remote Client Mode: Polling offloaded to {targetHost}:{targetPort}
          </div>

          <div class="form-group">
            <label for="adminToken">Admin Access Token (Optional)</label>
            <input
              id="adminToken"
              type="password"
              bind:value={adminToken}
              placeholder="Enter admin token to unlock edit permissions"
            />
            <span class="field-hint">
              Leave blank for standard read-only monitoring.
            </span>
          </div>
        {/if}
      </section>

      <section class="settings-section">
        <h3 class="section-title">Server Mode (Broadcast)</h3>
        <p class="section-description">
          Run an embedded server on this machine to broadcast monitored resources to other Caffeine clients.
        </p>

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={serverMode} />
          <span class="checkbox-text">Enable Caffeine Server Mode</span>
        </label>

        {#if serverMode}
          <div class="server-options">
            <div class="form-row">
              <div class="form-group flex-2">
                <label for="serverBind">Bind Address</label>
                <input
                  id="serverBind"
                  type="text"
                  bind:value={serverBind}
                  placeholder="0.0.0.0"
                />
              </div>
              <div class="form-group flex-1">
                <label for="serverPort">Server Port</label>
                <input
                  id="serverPort"
                  type="number"
                  bind:value={serverPort}
                  min="1"
                  max="65535"
                />
              </div>
            </div>
            <div class="token-management-section">
              <div class="token-section-header">
                <div>
                  <h4 class="token-section-title">Client Access Tokens</h4>
                  <p class="token-section-desc">
                    Assign labeled keys to authorized remote clients. For security, keys are generated with high entropy and only shown once.
                  </p>
                </div>
              </div>

              {#if tokenActionError}
                <div class="alert alert-error token-alert">
                  {tokenActionError}
                </div>
              {/if}

              {#if revealedSecret}
                <div class="revealed-secret-card">
                  <div class="revealed-secret-header">
                    <span class="revealed-badge">New Token for "{revealedSecret.label}"</span>
                    <button
                      type="button"
                      class="btn-close-secret"
                      onclick={() => (revealedSecret = null)}
                      title="Done"
                    >
                      ✕
                    </button>
                  </div>
                  <p class="revealed-warning">
                    ⚠️ <strong>Copy this key now.</strong> For your security, this key will never be displayed again. If forgotten, you must regenerate it.
                  </p>
                  <div class="revealed-key-box">
                    <code class="revealed-key">{revealedSecret.token}</code>
                    <button
                      type="button"
                      class="btn btn-primary btn-sm copy-btn"
                      onclick={copyRevealedSecret}
                    >
                      {secretCopied ? "Copied!" : "Copy Key"}
                    </button>
                  </div>
                </div>
              {/if}

              {#if serverTokens.length > 0}
                <div class="token-list-container">
                  <table class="token-table">
                    <thead>
                      <tr>
                        <th>Label</th>
                        <th>Key Preview</th>
                        <th>Created</th>
                        <th class="actions-header">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each serverTokens as token (token.id)}
                        <tr>
                          <td class="token-label-cell">
                            <strong>{token.label}</strong>
                          </td>
                          <td class="token-preview-cell">
                            <code>{token.token_preview}</code>
                          </td>
                          <td class="token-date-cell">
                            {new Date(token.created_at).toLocaleDateString()}
                          </td>
                          <td class="token-actions-cell">
                            <button
                              type="button"
                              class="btn btn-secondary btn-xs"
                              onclick={() => handleRegenerateToken(token)}
                              title="Regenerate this specific key"
                            >
                              Regenerate
                            </button>
                            <button
                              type="button"
                              class="btn btn-danger btn-xs"
                              onclick={() => handleDeleteToken(token)}
                              title="Revoke and delete this key"
                            >
                              Revoke
                            </button>
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {:else}
                <div class="empty-tokens-placeholder">
                  No access tokens configured yet. Create one below to grant remote clients edit permissions.
                </div>
              {/if}

              <div class="create-token-row">
                <input
                  type="text"
                  bind:value={newLabel}
                  placeholder="Token label (e.g. Josh's Laptop, Reception PC)"
                  class="token-label-input"
                  onkeydown={(e) => e.key === "Enter" && handleCreateToken()}
                />
                <button
                  type="button"
                  class="btn btn-secondary btn-sm"
                  onclick={handleCreateToken}
                  disabled={!newLabel.trim() || generatingToken}
                >
                  {generatingToken ? "Generating..." : "Generate Key"}
                </button>
              </div>
            </div>

            <p class="field-hint">
              Server mode reads and writes to system-wide machine storage and serves clients on the network.
            </p>
          </div>
        {/if}
      </section>

      <section class="settings-section">
        <h3 class="section-title">Local Log Retention</h3>
        <p class="section-description">
          Controls how much event history is kept in the local database replica.
        </p>

        <div class="form-row">
          <div class="form-group">
            <label for="retentionDays">Retention (Days)</label>
            <input
              id="retentionDays"
              type="number"
              bind:value={retentionDays}
              min="1"
              max="3650"
            />
          </div>
          <div class="form-group">
            <label for="maxLogEntries">Max Entries Per Entity</label>
            <input
              id="maxLogEntries"
              type="number"
              bind:value={maxLogEntries}
              min="50"
              max="100000"
            />
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">Appearance</h3>
        <div class="form-group">
          <label for="themeSelect">Theme</label>
          <select id="themeSelect" bind:value={theme}>
            <option value="system">System Default</option>
            <option value="dark">Dark Theme</option>
            <option value="light">Light Theme</option>
          </select>
        </div>
      </section>

      <div class="modal-actions">
        <button
          type="button"
          class="btn btn-secondary"
          onclick={handleClose}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="btn btn-primary"
          disabled={saving}
        >
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>
    </form>
  {/if}
</Modal>

<style>
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--color-text);
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--color-text-muted);
    cursor: pointer;
    line-height: 1;
    padding: 0 4px;
  }

  .close-button:hover {
    color: var(--color-text);
  }

  .settings-loading {
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .settings-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-bottom: 1rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .section-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .section-description {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    line-height: 1.35;
  }

  .section-description code {
    background-color: var(--color-surface-active);
    padding: 2px 4px;
    border-radius: 4px;
  }

  .btn-sm {
    padding: 7px 12px;
    font-size: 0.8125rem;
    white-space: nowrap;
  }

  .form-row {
    display: flex;
    gap: 12px;
  }

  .flex-1 {
    flex: 1;
  }

  .flex-2 {
    flex: 2;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text);
  }

  input[type="text"],
  input[type="number"],
  input[type="password"],
  select {
    background-color: var(--color-base);
    color: var(--color-text);
    border: solid var(--width-border) var(--color-border);
    border-radius: 4px;
    padding: 8px 10px;
    font-family: inherit;
    font-size: 0.9rem;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .field-hint {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .checkbox-label input {
    cursor: pointer;
    accent-color: var(--color-primary);
    width: 16px;
    height: 16px;
  }

  .server-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 12px;
    background-color: var(--color-surface-hover);
    border-radius: 6px;
  }

  .remote-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background-color: var(--color-surface-hover);
    border-radius: 4px;
    font-size: 0.8rem;
    color: var(--color-primary);
    font-weight: 500;
  }

  :global(.status-indicator) {
    color: var(--color-primary);
    flex-shrink: 0;
  }

  .alert {
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .alert-error {
    background-color: rgba(238, 102, 102, 0.15);
    color: var(--color-failure);
    border: solid var(--width-border) var(--color-failure);
  }

  .alert-success {
    background-color: rgba(102, 238, 102, 0.15);
    color: var(--color-success);
    border: solid var(--width-border) var(--color-success);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 0.5rem;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    font-family: inherit;
    border: solid var(--width-border) transparent;
  }

  .btn-secondary {
    background-color: transparent;
    color: var(--color-text);
    border-color: var(--color-border);
  }

  .btn-secondary:hover {
    background-color: var(--color-surface-hover);
  }

  .btn-primary {
    background-color: var(--color-primary);
    color: var(--color-base);
  }

  .btn-primary:hover {
    filter: brightness(1.1);
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .token-management-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 4px;
    padding-top: 8px;
    border-top: solid var(--width-border) var(--color-border);
  }

  .token-section-title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .token-section-desc {
    margin: 2px 0 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .revealed-secret-card {
    background-color: var(--color-surface-active);
    border: solid var(--width-border) var(--color-primary);
    border-radius: 6px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .revealed-secret-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .revealed-badge {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .btn-close-secret {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 2px 6px;
  }

  .btn-close-secret:hover {
    color: var(--color-text);
  }

  .revealed-warning {
    margin: 0;
    font-size: 0.775rem;
    color: var(--color-text);
    line-height: 1.3;
  }

  .revealed-key-box {
    display: flex;
    gap: 8px;
    align-items: center;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 4px;
    padding: 6px 10px;
  }

  .revealed-key {
    flex: 1;
    font-family: monospace;
    font-size: 0.8125rem;
    word-break: break-all;
    user-select: all;
    color: var(--color-primary);
  }

  .token-list-container {
    border: solid var(--width-border) var(--color-border);
    border-radius: 4px;
    overflow: hidden;
  }

  .token-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
    text-align: left;
  }

  .token-table th {
    background-color: var(--color-surface-active);
    color: var(--color-text-muted);
    font-weight: 500;
    padding: 6px 10px;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .token-table td {
    padding: 8px 10px;
    border-bottom: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .token-table tr:last-child td {
    border-bottom: none;
  }

  .token-preview-cell code {
    font-size: 0.75rem;
    background-color: var(--color-surface-hover);
    padding: 2px 4px;
    border-radius: 3px;
  }

  .token-actions-cell {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .empty-tokens-placeholder {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-style: italic;
    padding: 8px 0;
  }

  .create-token-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .token-label-input {
    flex: 1;
  }

  .btn-xs {
    padding: 4px 8px;
    font-size: 0.75rem;
    border-radius: 3px;
  }

  .btn-danger {
    background-color: transparent;
    color: var(--color-failure);
    border-color: rgba(238, 102, 102, 0.4);
  }

  .btn-danger:hover {
    background-color: rgba(238, 102, 102, 0.15);
    border-color: var(--color-failure);
  }
</style>
