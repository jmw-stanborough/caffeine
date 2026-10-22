<script lang="ts">
  import { formatTimestamp } from "$lib/format";

  export interface ConnectionLogItem {
    id: number;
    event_type: string;
    latency_ms?: number;
    timestamp: string;
  }

  let {
    logs,
    loading,
    error,
    onclose,
  }: {
    logs: ConnectionLogItem[];
    loading: boolean;
    error: string | null;
    onclose: () => void;
  } = $props();
</script>

<div class="history-body">
  {#if loading}
    <p class="history-state">Loading history...</p>
  {:else if error}
    <p class="history-state error">{error}</p>
  {:else if logs.length === 0}
    <p class="history-state">No connection events recorded yet.</p>
  {:else}
    <div class="log-list">
      {#each logs as log (log.id)}
        <div class="log-item">
          <span
            class="log-badge"
            class:connected={log.event_type === "connected"}
            class:disconnected={log.event_type !== "connected"}
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
    <button type="button" class="btn-cancel" onclick={onclose}>
      Close
    </button>
  </div>
</div>

<style>
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
</style>
