<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import { formatUptime } from "./metrics";
  import type { EndpointMachineInfo } from "./types";
  import BatteryCard from "./cards/battery-card.svelte";
  import CpuCard from "./cards/cpu-card.svelte";
  import GpuCard from "./cards/gpu-card.svelte";
  import MemoryCard from "./cards/memory-card.svelte";
  import MotherboardCard from "./cards/motherboard-card.svelte";
  import NetworkCard from "./cards/network-card.svelte";
  import StorageCard from "./cards/storage-card.svelte";
  import SystemCard from "./cards/system-card.svelte";
  import ThermalCard from "./cards/thermal-card.svelte";

  interface Props {
    machine: EndpointMachineInfo | null;
    isLoading: boolean;
    errorMessage: string | null;
    onRefresh: () => void;
  }

  let { machine, isLoading, errorMessage, onRefresh }: Props = $props();
</script>

<div class="hardware-module">
  <div class="page-header">
    <div class="header-left">
      <h2 class="page-title">System & Hardware</h2>
      {#if machine}
        <span class="uptime-badge" title="System Uptime">
          Up: {formatUptime(machine.system.uptime_seconds)}
        </span>
        {#if machine.system.fast_startup_enabled}
          <span
            class="fast-startup-warning"
            role="status"
            title="Fast startup is turned on; system uptime is not reliable"
          >
            <AlertTriangle size={13} />
            <span>Fast startup is turned on; system uptime is not reliable</span>
          </span>
        {/if}
      {/if}
    </div>
    <button
      class="btn-refresh"
      onclick={onRefresh}
      disabled={isLoading}
      title="Refresh hardware metrics"
      aria-label="Refresh hardware metrics"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="15"
        height="15"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class:spin={isLoading}
      >
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
        <path d="M16 21h5v-5" />
      </svg>
      <span>{isLoading ? "Reading..." : "Refresh"}</span>
    </button>
  </div>

  {#if errorMessage}
    <div class="error-banner">
      <span>{errorMessage}</span>
      <button class="btn-retry" onclick={onRefresh}>Retry</button>
    </div>
  {/if}

  {#if !machine && isLoading}
    <div class="skeleton-container">
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
    </div>
  {:else if machine}
    <div class="hardware-content">
      <SystemCard system={machine.system} />
      <CpuCard cpu={machine.cpu} />
      <MemoryCard memory={machine.memory} />
      <StorageCard disks={machine.disks} />
      {#if machine.temperatures.length > 0}
        <ThermalCard temperatures={machine.temperatures} />
      {/if}
      {#if machine.gpus.length > 0}
        <GpuCard gpus={machine.gpus} />
      {/if}
      {#if machine.motherboard}
        <MotherboardCard motherboard={machine.motherboard} />
      {/if}
      {#if machine.networks.length > 0}
        <NetworkCard networks={machine.networks} />
      {/if}
      {#if machine.battery && machine.battery.has_battery}
        <BatteryCard battery={machine.battery} />
      {/if}
    </div>
  {/if}
</div>

<style>
  .hardware-module {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    flex-shrink: 0;
    gap: 0.75rem;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .page-title {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .uptime-badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 999px;
    background-color: var(--color-surface);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .fast-startup-warning {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.55rem;
    border-radius: 999px;
    background-color: color-mix(in srgb, var(--color-warning) 14%, transparent);
    color: var(--color-warning);
    border: solid var(--width-border) color-mix(in srgb, var(--color-warning) 35%, transparent);
  }

  .btn-refresh {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.35rem 0.75rem;
    border-radius: 0.35rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .btn-refresh:hover:not(:disabled) {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
  }

  .btn-refresh:disabled {
    opacity: 0.65;
    cursor: not-allowed;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.625rem 0.875rem;
    background-color: color-mix(in srgb, var(--color-failure) 15%, var(--color-surface));
    border: solid var(--width-border) var(--color-failure);
    border-radius: 0.375rem;
    color: var(--color-failure);
    font-size: 0.85rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    flex-shrink: 0;
  }

  .btn-retry {
    background-color: transparent;
    border: solid 1px var(--color-failure);
    color: var(--color-failure);
    border-radius: 0.25rem;
    padding: 0.2rem 0.55rem;
    font-size: 0.75rem;
    font-weight: 700;
    cursor: pointer;
  }

  .btn-retry:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .hardware-content {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    padding-bottom: 0.5rem;
  }

  .skeleton-container {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .skeleton-card {
    height: 120px;
    border-radius: 0.5rem;
    background: linear-gradient(
      90deg,
      var(--color-surface) 25%,
      var(--color-surface-hover) 50%,
      var(--color-surface) 75%
    );
    background-size: 200% 100%;
    animation: skeleton-pulse 1.5s infinite;
  }

  @keyframes skeleton-pulse {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
