<script lang="ts">
  import { activateOnEnterOrSpace, formatRelativeTime } from "$lib/format";
  import { Circle } from "$lib/components/icons";
  import type { HostState, Server } from "./types";

  let {
    server,
    status = "checking",
    latencyMs = undefined,
    lastEvent = undefined,
    lastEventAt = undefined,
    now = Date.now(),
    nocMode = false,
    onclick,
    onterminal,
    onremotedesktop,
  }: {
    server: Server;
    status?: HostState["status"];
    latencyMs?: number;
    lastEvent?: HostState["lastEvent"];
    lastEventAt?: string;
    now?: number;
    nocMode?: boolean;
    onclick?: () => void;
    onterminal?: (server: Server) => void;
    onremotedesktop?: (server: Server) => void;
  } = $props();

  let relativeTime = $derived(formatRelativeTime(lastEventAt, now));
</script>

<div
  class="server-card {status}"
  class:noc={nocMode}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={(e) => activateOnEnterOrSpace(e, onclick)}
>
  <div class="card-main">
    <div class="card-title-row">
      {#if !nocMode}
        <Circle
          size={10}
          class="status-dot {status}"
          aria-hidden="true"
        />
      {/if}
      <h2 class="card-label">{server.label}</h2>
      {#if server.agent && !nocMode}
        <span class="badge-agent">Agent :{server.agent.port}</span>
      {/if}
    </div>

    {#if !nocMode}
      <p class="card-description">{server.description ?? "No description provided."}</p>
    {/if}
  </div>

  <div class="card-footer">
    <div class="footer-left">
      <span class="card-host">{server.host}</span>
      {#if lastEvent && lastEventAt}
        <span class="card-since">
          {lastEvent === "connected" ? "Connected" : "Disconnected"} {relativeTime}
        </span>
      {/if}
    </div>

    <div class="footer-right">
      {#if !nocMode}
        <div class="card-actions">
          <button
            type="button"
            class="btn-card-action"
            onclick={(e) => {
              e.stopPropagation();
              onterminal?.(server);
            }}
            title={status === "offline"
              ? "Terminal (Host is offline)"
              : `Open Terminal (${server.ssh_port ?? 22})`}
            aria-label="Open SSH Terminal"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" x2="20" y1="19" y2="19" />
            </svg>
          </button>

          <button
            type="button"
            class="btn-card-action"
            onclick={(e) => {
              e.stopPropagation();
              onremotedesktop?.(server);
            }}
            title={status === "offline"
              ? "Remote Desktop (Host is offline)"
              : `Launch Remote Desktop (${server.rdp_port ?? 3389})`}
            aria-label="Launch Remote Desktop"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
              <line x1="8" y1="21" x2="16" y2="21" />
              <line x1="12" y1="17" x2="12" y2="21" />
            </svg>
          </button>
        </div>
      {/if}

      <span class="card-latency">
        {#if latencyMs !== undefined}
          {latencyMs} ms
        {:else if status === "checking"}
          ...
        {:else}
          —
        {/if}
      </span>
    </div>
  </div>
</div>

<style>
  .server-card {
    border: solid var(--width-border) var(--status-color, var(--color-border));
    border-radius: 0.5rem;
    padding: 1rem;
    min-height: var(--card-min-height, 8.5rem);
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 0.75rem;
    cursor: pointer;
    user-select: none;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .server-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .server-card.noc {
    cursor: default;
  }

  .server-card.noc .card-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }

  .server-card.noc .card-title-row {
    justify-content: center;
  }

  .server-card.noc .card-label {
    font-size: calc(1.75rem * var(--card-font-scale, 1));
    font-weight: 700;
    text-align: center;
  }

  .server-card.online {
    --status-color: var(--color-success);
  }

  .server-card.degraded {
    --status-color: var(--color-warning);
  }

  .server-card.offline {
    --status-color: var(--color-failure);
  }

  .server-card:is(.online, .degraded, .offline) {
    background-color: color-mix(in srgb, var(--status-color) 32%, var(--color-surface));
  }

  .server-card:is(.online, .degraded, .offline):hover {
    background-color: color-mix(in srgb, var(--status-color) 42%, var(--color-surface));
  }

  .server-card.checking {
    --status-color: var(--color-text-muted);
    background-color: color-mix(in srgb, var(--color-surface) 60%, transparent);
    border-color: var(--color-border);
  }

  .server-card.checking:hover {
    background-color: color-mix(in srgb, var(--color-surface) 85%, transparent);
    border-color: var(--color-text-muted);
  }

  .card-main {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .card-label {
    margin: 0;
    font-size: calc(1rem * var(--card-font-scale, 1));
    font-weight: 600;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge-agent {
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    padding: 0.15rem 0.35rem;
    border-radius: 0.25rem;
    background-color: color-mix(
      in srgb,
      var(--status-color, var(--color-primary)) 20%,
      transparent
    );
    color: var(--status-color, var(--color-primary));
    border: solid var(--width-border)
      color-mix(
        in srgb,
        var(--status-color, var(--color-primary)) 40%,
        transparent
      );
    margin-left: auto;
  }

  .card-description {
    margin: 0;
    font-size: calc(0.8rem * var(--card-font-scale, 1));
    color: var(--color-text);
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .card-footer {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    font-size: calc(0.75rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    margin-top: auto;
  }

  .footer-left {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .btn-card-action {
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: color-mix(in srgb, var(--color-surface) 70%, transparent);
    border: solid 1px var(--color-border);
    color: var(--color-text);
    padding: 0.25rem;
    border-radius: 0.25rem;
    cursor: pointer;
    transition: background-color 0.12s ease, border-color 0.12s ease, color 0.12s ease;
  }

  .btn-card-action:hover {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .card-host {
    font-family: monospace;
  }

  .card-since {
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    opacity: 0.85;
  }

  .card-latency {
    font-variant-numeric: tabular-nums;
  }
</style>
