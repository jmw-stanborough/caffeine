<script lang="ts">
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
  }: {
    server: Server;
    status?: HostState["status"];
    latencyMs?: number;
    lastEvent?: HostState["lastEvent"];
    lastEventAt?: string;
    now?: number;
    nocMode?: boolean;
    onclick?: () => void;
  } = $props();

  function formatRelativeTime(dateString?: string, currentTime: number = Date.now()): string {
    if (!dateString) return "";
    const parsed = new Date(dateString);
    if (isNaN(parsed.getTime())) return "";

    const diffSec = Math.floor((currentTime - parsed.getTime()) / 1000);
    if (diffSec < 60) return "just now";
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
    return `${Math.floor(diffSec / 86400)}d ago`;
  }

  let relativeTime = $derived(formatRelativeTime(lastEventAt, now));

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.key === "Enter" || e.key === " ") && onclick) {
      e.preventDefault();
      onclick();
    }
  }
</script>

<div
  class="server-card {status}"
  class:noc={nocMode}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={handleKeyDown}
>
  <div class="card-main">
    <div class="card-title-row">
      <span class="status-dot" aria-hidden="true"></span>
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
    min-height: var(--card-min-height, 8.5rem);
    padding: 1rem;
  }

  .server-card.online {
    --status-color: var(--color-primary);
    background-color: color-mix(in srgb, var(--status-color) 18%, var(--color-surface));
  }

  .server-card.online:hover {
    background-color: color-mix(in srgb, var(--status-color) 28%, var(--color-surface));
  }

  .server-card.degraded {
    --status-color: var(--color-secondary);
    background-color: color-mix(in srgb, var(--status-color) 18%, var(--color-surface));
  }

  .server-card.degraded:hover {
    background-color: color-mix(in srgb, var(--status-color) 28%, var(--color-surface));
  }

  .server-card.offline {
    --status-color: var(--color-error);
    background-color: color-mix(in srgb, var(--status-color) 18%, var(--color-surface));
  }

  .server-card.offline:hover {
    background-color: color-mix(in srgb, var(--status-color) 28%, var(--color-surface));
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

  .status-dot {
    width: 0.55rem;
    height: 0.55rem;
    border-radius: 50%;
    background-color: var(--status-color, var(--color-border));
    flex-shrink: 0;
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
    background-color: color-mix(in srgb, var(--status-color, var(--color-primary)) 20%, transparent);
    color: var(--status-color, var(--color-primary));
    border: solid var(--width-border) color-mix(in srgb, var(--status-color, var(--color-primary)) 40%, transparent);
    margin-left: auto;
  }

  .card-description {
    margin: 0;
    font-size: calc(0.8rem * var(--card-font-scale, 1));
    color: var(--color-text-muted);
    overflow: hidden;
    line-clamp: 2;
    display: box;
    box-orient: vertical;
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
