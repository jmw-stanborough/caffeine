<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toErrorMessage } from "$lib/errors";
  import type { GeneratedTokenResponse, ServerTokenPublic } from "./types";

  let {
    tokens = $bindable<ServerTokenPublic[]>([]),
  }: {
    tokens: ServerTokenPublic[];
  } = $props();

  let newLabel = $state("");
  let generatingToken = $state(false);
  let tokenActionError = $state<string | null>(null);
  let revealedSecret = $state<GeneratedTokenResponse | null>(null);
  let secretCopied = $state(false);

  async function loadServerTokens() {
    try {
      tokens = await invoke<ServerTokenPublic[]>("get_server_tokens_cmd");
    } catch (err) {
      tokenActionError = toErrorMessage(err);
    }
  }

  async function handleCreateToken() {
    if (!newLabel.trim()) return;
    generatingToken = true;
    tokenActionError = null;
    try {
      const generated = await invoke<GeneratedTokenResponse>(
        "create_server_token_cmd",
        { label: newLabel.trim() },
      );
      revealedSecret = generated;
      newLabel = "";
      await loadServerTokens();
    } catch (err) {
      tokenActionError = toErrorMessage(err);
    } finally {
      generatingToken = false;
    }
  }

  async function handleRegenerateToken(token: ServerTokenPublic) {
    if (
      !confirm(
        `Regenerating the token for "${token.label}" will immediately revoke the previous key. Any remote clients using it will lose access until updated. Proceed?`,
      )
    ) {
      return;
    }
    tokenActionError = null;
    try {
      const regenerated = await invoke<GeneratedTokenResponse>(
        "regenerate_server_token_cmd",
        { id: token.id },
      );
      revealedSecret = regenerated;
      await loadServerTokens();
    } catch (err) {
      tokenActionError = toErrorMessage(err);
    }
  }

  async function handleDeleteToken(token: ServerTokenPublic) {
    if (
      !confirm(
        `Are you sure you want to revoke the token for "${token.label}"? Any remote clients using it will immediately be rejected.`,
      )
    ) {
      return;
    }
    tokenActionError = null;
    try {
      await invoke("delete_server_token_cmd", { id: token.id });
      await loadServerTokens();
    } catch (err) {
      tokenActionError = toErrorMessage(err);
    }
  }

  async function copyRevealedSecret() {
    if (!revealedSecret) return;
    tokenActionError = null;
    try {
      await navigator.clipboard.writeText(revealedSecret.token);
      secretCopied = true;
      setTimeout(() => {
        secretCopied = false;
      }, 2500);
    } catch (err) {
      tokenActionError = `Failed to copy token: ${toErrorMessage(err)}`;
    }
  }
</script>

<div class="token-management-section">
  <div>
    <h4 class="token-section-title">Client Access Tokens</h4>
    <p class="token-section-desc">
      Assign labeled keys to authorized remote clients. For security, keys are generated with high entropy and only shown once.
    </p>
  </div>

  {#if tokenActionError}
    <div class="error-banner">{tokenActionError}</div>
  {/if}

  {#if revealedSecret}
    <div class="revealed-secret-card">
      <div class="revealed-secret-header">
        <span class="revealed-badge">New Token for "{revealedSecret.label}"</span>
        <button
          type="button"
          class="btn-close"
          onclick={() => (revealedSecret = null)}
          title="Done"
          aria-label="Dismiss generated token"
        >
          ✕
        </button>
      </div>
      <p class="revealed-warning">
        <strong>Copy this key now.</strong> For your security, this key will never be displayed again. If forgotten, you must regenerate it.
      </p>
      <div class="revealed-key-box">
        <code class="revealed-key">{revealedSecret.token}</code>
        <button
          type="button"
          class="btn-save btn-compact"
          onclick={copyRevealedSecret}
        >
          {secretCopied ? "Copied!" : "Copy Key"}
        </button>
      </div>
    </div>
  {/if}

  {#if tokens.length > 0}
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
          {#each tokens as token (token.id)}
            <tr>
              <td><strong>{token.label}</strong></td>
              <td class="token-preview-cell"><code>{token.token_preview}</code></td>
              <td>{new Date(token.created_at).toLocaleDateString()}</td>
              <td class="token-actions-cell">
                <button
                  type="button"
                  class="btn-cancel btn-xs"
                  onclick={() => handleRegenerateToken(token)}
                  title="Regenerate this specific key"
                >
                  Regenerate
                </button>
                <button
                  type="button"
                  class="btn-delete btn-xs"
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
      aria-label="New client access token label"
      onkeydown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          handleCreateToken();
        }
      }}
    />
    <button
      type="button"
      class="btn-cancel btn-compact"
      onclick={handleCreateToken}
      disabled={!newLabel.trim() || generatingToken}
    >
      {generatingToken ? "Generating..." : "Generate Key"}
    </button>
  </div>
</div>

<style>
  .token-management-section {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    margin-top: 0.25rem;
    padding-top: 0.65rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .token-section-title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .token-section-desc {
    margin: 0.15rem 0 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .revealed-secret-card {
    background-color: var(--color-surface-active);
    border: solid var(--width-border) var(--color-primary);
    border-radius: 0.375rem;
    padding: 0.65rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
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

  .revealed-warning {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color-text);
    line-height: 1.3;
  }

  .revealed-key-box {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.4rem 0.65rem;
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
    border-radius: 0.25rem;
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
    padding: 0.4rem 0.65rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .actions-header {
    text-align: right;
  }

  .token-table td {
    padding: 0.5rem 0.65rem;
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
    gap: 0.4rem;
    justify-content: flex-end;
  }

  .empty-tokens-placeholder {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-style: italic;
    padding: 0.4rem 0;
  }

  .create-token-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .token-label-input {
    flex: 1;
    min-width: 0;
    background-color: var(--color-base);
    color: var(--color-text);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.45rem 0.65rem;
    font-family: inherit;
    font-size: 0.85rem;
    outline: none;
  }

  .token-label-input:focus {
    border-color: var(--color-primary);
  }

  .btn-compact {
    padding: 0.45rem 0.75rem;
    font-size: 0.8125rem;
    white-space: nowrap;
  }

  .btn-compact:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .btn-xs {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 500;
  }
</style>
