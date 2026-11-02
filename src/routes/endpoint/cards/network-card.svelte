<script lang="ts">
  import { Circle } from "$lib/icons";
  import { formatBytes } from "../metrics";
  import type { NetworkInterfaceInfo } from "../types";

  interface Props {
    networks: NetworkInterfaceInfo[];
  }

  let { networks }: Props = $props();
</script>

<section class="section-card">
  <div class="card-header">
    <div class="header-with-icon">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="2" x2="22" y1="12" y2="12" />
        <path
          d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
        />
      </svg>
      <h3 class="card-title">Network Adapters</h3>
    </div>
  </div>

  <div class="network-list">
    {#each networks as net}
      <div class="network-item">
        <div class="network-header-row">
          <span class="net-name">{net.name}</span>
          <Circle
            size={8}
            class="net-status-dot {net.is_up ? 'active' : ''}"
            title={net.is_up ? "Active" : "Inactive"}
          />
        </div>
        {#if net.mac_address}
          <div class="net-row">
            <span class="spec-label">MAC:</span>
            <span class="net-val mono">{net.mac_address}</span>
          </div>
        {/if}
        {#if net.ip_v4.length > 0}
          <div class="net-row">
            <span class="spec-label">IPv4:</span>
            <span class="net-val mono">{net.ip_v4.join(", ")}</span>
          </div>
        {/if}
        <div class="net-row transfer-stats">
          <span>Down: {formatBytes(net.bytes_received)}</span>
          <span>Up: {formatBytes(net.bytes_transmitted)}</span>
        </div>
      </div>
    {/each}
  </div>
</section>

<style>
  .section-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 1.15rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header-with-icon {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-primary);
  }

  .card-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .network-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .network-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.6rem 0.75rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .network-header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .net-name {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text);
  }

  :global(.net-status-dot) {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  :global(.net-status-dot.active) {
    color: var(--color-success);
  }

  .net-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
  }

  .spec-label {
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .net-val {
    color: var(--color-text);
  }

  .mono {
    font-family: monospace;
  }

  .transfer-stats {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    gap: 1rem;
    margin-top: 0.15rem;
  }
</style>
