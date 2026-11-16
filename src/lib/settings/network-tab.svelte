<script lang="ts">
  import StatusDot from "$lib/components/status-dot.svelte";
  import { isRemoteHost } from "$lib/client-mode.svelte";
  import ServerTokensSection from "./server-tokens-section.svelte";
  import type { ServerTokenPublic } from "./types";

  let {
    targetHost = $bindable("127.0.0.1"),
    targetPort = $bindable(6310),
    adminToken = $bindable(""),
    serverMode = $bindable(false),
    serverBind = $bindable("0.0.0.0"),
    serverPort = $bindable(6310),
    serverTokens = $bindable<ServerTokenPublic[]>([]),
  }: {
    targetHost: string;
    targetPort: number;
    adminToken: string;
    serverMode: boolean;
    serverBind: string;
    serverPort: number;
    serverTokens: ServerTokenPublic[];
  } = $props();

  let isRemoteClient = $derived(isRemoteHost(targetHost));
</script>

<section class="network-subsection">
  <h3 class="subsection-title">Connection Target</h3>
  <p class="settings-description">
    Defaults to <code>127.0.0.1</code> (self-serving client). Point to an IP or hostname to consume from a remote Caffeine instance.
  </p>

  <div class="settings-row">
    <label class="form-field flex-2" for="targetHost">
      <span>Target Host / IP</span>
      <input
        id="targetHost"
        type="text"
        bind:value={targetHost}
        placeholder="127.0.0.1 or server.domain.lan"
      />
    </label>
    <label class="form-field" for="targetPort">
      <span>Target Port</span>
      <input
        id="targetPort"
        type="number"
        bind:value={targetPort}
        min="1"
        max="65535"
      />
    </label>
  </div>

  {#if isRemoteClient}
    <div class="remote-badge">
      <StatusDot status="success" size={8} />
      Remote Client Mode: Polling offloaded to {targetHost}:{targetPort}
    </div>

    <label class="form-field" for="adminToken">
      <span>Admin Access Token (Optional)</span>
      <input
        id="adminToken"
        type="password"
        bind:value={adminToken}
        placeholder="Enter admin token to unlock edit permissions"
      />
      <span class="field-hint">
        Leave blank for standard read-only monitoring.
      </span>
    </label>
  {/if}
</section>

<section class="network-subsection">
  <h3 class="subsection-title">Server Mode (Broadcast)</h3>
  <p class="settings-description">
    Run an embedded server on this machine to broadcast monitored resources to other Caffeine clients.
  </p>

  <label class="checkbox-label">
    <input type="checkbox" bind:checked={serverMode} />
    <span>Enable Caffeine Server Mode</span>
  </label>

  {#if serverMode}
    <div class="server-options">
      <div class="settings-row">
        <label class="form-field flex-2" for="serverBind">
          <span>Bind Address</span>
          <input
            id="serverBind"
            type="text"
            bind:value={serverBind}
            placeholder="0.0.0.0"
          />
        </label>
        <label class="form-field" for="serverPort">
          <span>Server Port</span>
          <input
            id="serverPort"
            type="number"
            bind:value={serverPort}
            min="1"
            max="65535"
          />
        </label>
      </div>

      <ServerTokensSection bind:tokens={serverTokens} />

      <span class="field-hint">
        Server mode reads and writes to system-wide machine storage and serves clients on the network.
      </span>
    </div>
  {/if}
</section>

<style>
  .network-subsection {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .network-subsection + .network-subsection {
    padding-top: 0.85rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .subsection-title {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .remote-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.65rem;
    background-color: var(--color-surface-hover);
    border-radius: 0.25rem;
    font-size: 0.8rem;
    color: var(--color-primary);
    font-weight: 500;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-text);
  }

  .checkbox-label input {
    cursor: pointer;
    accent-color: var(--color-primary);
    width: 1rem;
    height: 1rem;
  }

  .server-options {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 0.75rem;
    background-color: var(--color-surface-hover);
    border-radius: 0.375rem;
  }
</style>
