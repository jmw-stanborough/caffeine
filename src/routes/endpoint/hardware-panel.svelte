<script lang="ts">
  import ProgressBar from "./progress-bar.svelte";
  import {
    formatBytes,
    formatUptime,
    getTemperatureColor,
  } from "./metrics";
  import type { EndpointMachineInfo } from "./types";

  interface Props {
    machine: EndpointMachineInfo | null;
    isLoading: boolean;
    errorMessage: string | null;
    onRefresh: () => void;
  }

  let {
    machine,
    isLoading,
    errorMessage,
    onRefresh,
  }: Props = $props();
</script>

<div class="hardware-panel">
  <div class="panel-header">
    <div class="header-left">
      <h2 class="panel-title">System & Hardware</h2>
      {#if machine}
        <span class="uptime-badge" title="System Uptime">
          Up: {formatUptime(machine.system.uptime_seconds)}
        </span>
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
    <div class="panel-scroll-content">
      <!-- System Identity Card -->
      <section class="section-card system-card">
        <div class="card-title-row">
          <div class="system-main">
            <span class="host-name">{machine.system.computer_name}</span>
            <span class="domain-tag">{machine.system.domain_or_workgroup}</span>
          </div>
          <span class="arch-badge">{machine.system.architecture}</span>
        </div>
        <div class="os-info-row">
          <span class="os-name">{machine.system.os_name} {machine.system.os_version}</span>
          {#if machine.system.build_number}
            <span class="build-tag">Build {machine.system.build_number}</span>
          {/if}
        </div>
      </section>

      <!-- Primary Hardware: CPU -->
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
              <rect width="16" height="16" x="4" y="4" rx="2" />
              <rect width="6" height="6" x="9" y="9" rx="1" />
              <path d="M15 2v2" /><path d="M15 20v2" /><path d="M2 15h2" /><path d="M2 9h2" />
              <path d="M20 15h2" /><path d="M20 9h2" /><path d="M9 2v2" /><path d="M9 20v2" />
            </svg>
            <h3 class="card-title">Processor (CPU)</h3>
          </div>
          {#if machine.cpu.temperature_celsius != null}
            {@const c = machine.cpu.temperature_celsius}
            <span
              class="temp-chip"
              style:color={getTemperatureColor(c)}
              title="CPU Temperature"
            >
              {c.toFixed(1)}°C
            </span>
          {/if}
        </div>

        <div class="item-lead">{machine.cpu.model_name}</div>

        <div class="metric-specs-grid">
          <div class="spec-cell">
            <span class="spec-label">Cores</span>
            <span class="spec-value">{machine.cpu.physical_cores}P / {machine.cpu.logical_cores}T</span>
          </div>
          <div class="spec-cell">
            <span class="spec-label">Frequency</span>
            <span class="spec-value">{machine.cpu.frequency_mhz} MHz</span>
          </div>
          <div class="spec-cell">
            <span class="spec-label">Utilization</span>
            <span class="spec-value">{machine.cpu.global_usage_percent.toFixed(1)}%</span>
          </div>
        </div>

        <div class="progress-block">
          <ProgressBar
            value={machine.cpu.global_usage_percent}
            label="Overall CPU Load"
            showLabel={true}
            height="9px"
          />
        </div>

        {#if machine.cpu.core_usages.length > 0}
          <div class="cores-subgrid">
            {#each machine.cpu.core_usages as usage, i}
              <div class="mini-core-cell" title={`Core ${i + 1}: ${usage.toFixed(0)}%`}>
                <span class="core-num">C{i + 1}</span>
                <div class="mini-core-bar">
                  <div
                    class="mini-core-fill"
                    style:height={`${Math.min(100, usage)}%`}
                    style:background-color={getTemperatureColor(usage < 70 ? 50 : usage < 85 ? 75 : 85)}
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Primary Hardware: Memory (RAM) -->
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
              <path d="M6 19v-3" /><path d="M10 19v-3" /><path d="M14 19v-3" /><path d="M18 19v-3" />
              <rect x="2" y="5" width="20" height="10" rx="2" />
            </svg>
            <h3 class="card-title">Memory (RAM)</h3>
          </div>
          <span class="fraction-label">
            {formatBytes(machine.memory.used_bytes)} / {formatBytes(machine.memory.total_bytes)}
          </span>
        </div>

        <div class="progress-block">
          <ProgressBar
            value={machine.memory.usage_percent}
            label="Memory Allocation"
            showLabel={true}
            height="10px"
          />
        </div>

        <div class="metric-specs-grid">
          <div class="spec-cell">
            <span class="spec-label">Total</span>
            <span class="spec-value">{formatBytes(machine.memory.total_bytes)}</span>
          </div>
          <div class="spec-cell">
            <span class="spec-label">In Use</span>
            <span class="spec-value">{formatBytes(machine.memory.used_bytes)}</span>
          </div>
          <div class="spec-cell">
            <span class="spec-label">Available</span>
            <span class="spec-value">{formatBytes(machine.memory.available_bytes)}</span>
          </div>
        </div>
      </section>

      <!-- Primary Hardware: Secondary Storage Devices -->
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
              <line x1="22" x2="2" y1="12" y2="12" />
              <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />
              <line x1="6" x2="6.01" y1="16" y2="16" />
              <line x1="10" x2="10.01" y1="16" y2="16" />
            </svg>
            <h3 class="card-title">Secondary Storage Devices</h3>
          </div>
          <span class="storage-count-tag">{machine.disks.length} {machine.disks.length === 1 ? "drive" : "drives"}</span>
        </div>

        <div class="disks-list">
          {#each machine.disks as disk}
            <div class="disk-item">
              <div class="disk-meta-row">
                <div class="disk-name-wrap">
                  <span class="disk-mount">{disk.mount_point}</span>
                  {#if disk.name && disk.name !== disk.mount_point}
                    <span class="disk-label">{disk.name}</span>
                  {/if}
                  {#if disk.file_system}
                    <span class="fs-badge">{disk.file_system}</span>
                  {/if}
                </div>
                <span class="disk-usage-fraction">
                  {formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)}
                </span>
              </div>

              <ProgressBar
                value={disk.usage_percent}
                showLabel={true}
                height="8px"
              />

              <div class="disk-free-row">
                <span class="free-text">{formatBytes(disk.available_bytes)} free</span>
                {#if disk.is_removable}
                  <span class="removable-badge">Removable</span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>

      <!-- Thermal Readings (Temperatures) -->
      {#if machine.temperatures.length > 0}
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
                <path d="M14 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z" />
              </svg>
              <h3 class="card-title">Thermal Sensors</h3>
            </div>
          </div>

          <div class="temp-readings-grid">
            {#each machine.temperatures as temp}
              <div class="temp-reading-card">
                <span class="temp-label">{temp.label}</span>
                <span
                  class="temp-value-metric"
                  style:color={getTemperatureColor(temp.celsius)}
                >
                  {temp.celsius.toFixed(1)}°C
                </span>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Graphics (GPU) -->
      {#if machine.gpus.length > 0}
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
                <rect width="20" height="14" x="2" y="3" rx="2" />
                <line x1="8" x2="16" y1="21" y2="21" />
                <line x1="12" x2="12" y1="17" y2="21" />
              </svg>
              <h3 class="card-title">Graphics (GPU)</h3>
            </div>
          </div>

          <div class="gpu-list">
            {#each machine.gpus as gpu}
              <div class="gpu-item">
                <div class="gpu-name-row">
                  <span class="gpu-name">{gpu.name}</span>
                  {#if gpu.vram_bytes > 0}
                    <span class="vram-tag">{formatBytes(gpu.vram_bytes)} VRAM</span>
                  {/if}
                </div>
                {#if gpu.driver_version}
                  <div class="gpu-driver-row">
                    <span class="spec-label">Driver:</span>
                    <span class="driver-num">{gpu.driver_version}</span>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Motherboard & BIOS -->
      {#if machine.motherboard}
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
                <rect x="2" y="2" width="20" height="20" rx="2" />
                <rect x="6" y="6" width="4" height="4" />
                <rect x="14" y="6" width="4" height="4" />
                <rect x="10" y="14" width="4" height="4" />
              </svg>
              <h3 class="card-title">Motherboard & Firmware</h3>
            </div>
          </div>

          <div class="metric-specs-grid">
            <div class="spec-cell">
              <span class="spec-label">Board</span>
              <span class="spec-value">{machine.motherboard.manufacturer} {machine.motherboard.product}</span>
            </div>
            <div class="spec-cell">
              <span class="spec-label">BIOS</span>
              <span class="spec-value">{machine.motherboard.bios_version}</span>
            </div>
            {#if machine.motherboard.bios_date}
              <div class="spec-cell">
                <span class="spec-label">Date</span>
                <span class="spec-value">{machine.motherboard.bios_date}</span>
              </div>
            {/if}
          </div>
        </section>
      {/if}

      <!-- Network Adapters -->
      {#if machine.networks.length > 0}
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
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
              <h3 class="card-title">Network Adapters</h3>
            </div>
          </div>

          <div class="network-list">
            {#each machine.networks as net}
              <div class="network-item">
                <div class="network-header-row">
                  <span class="net-name">{net.name}</span>
                  <span class="net-status-dot" class:active={net.is_up} title={net.is_up ? "Active" : "Inactive"}></span>
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
      {/if}

      <!-- Battery / Power -->
      {#if machine.battery && machine.battery.has_battery}
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
                <rect width="16" height="10" x="2" y="7" rx="2" ry="2" />
                <line x1="22" x2="22" y1="11" y2="13" />
              </svg>
              <h3 class="card-title">Power & Battery</h3>
            </div>
            <span class="battery-status-tag">
              {machine.battery.is_charging ? "Charging" : "Discharging"}
            </span>
          </div>

          <ProgressBar
            value={machine.battery.charge_percent}
            label="Battery Level"
            showLabel={true}
            height="8px"
          />
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .hardware-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.65rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .panel-title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .uptime-badge {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 999px;
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .btn-refresh {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background-color: var(--color-base);
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

  .panel-scroll-content {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 0.85rem;
    overflow-y: auto;
    flex: 1;
    background-color: var(--color-base);
  }

  .section-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem 1.15rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
  }

  .system-card {
    background-color: color-mix(in srgb, var(--color-surface) 90%, var(--color-primary) 10%);
    border-color: color-mix(in srgb, var(--color-border) 80%, var(--color-primary) 20%);
  }

  .card-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .system-main {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .host-name {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .domain-tag {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .arch-badge {
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 700;
    color: var(--color-primary);
  }

  .os-info-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .build-tag {
    font-size: 0.75rem;
    padding: 0.1rem 0.4rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
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

  .temp-chip {
    font-size: 0.85rem;
    font-weight: 700;
    font-family: monospace;
  }

  .fraction-label,
  .storage-count-tag {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .item-lead {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .metric-specs-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.6rem;
  }

  .spec-cell {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.5rem 0.65rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .spec-label {
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .spec-value {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .progress-block {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .cores-subgrid {
    display: flex;
    gap: 0.35rem;
    overflow-x: auto;
    padding-top: 0.35rem;
  }

  .mini-core-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 18px;
  }

  .core-num {
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .mini-core-bar {
    width: 8px;
    height: 32px;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    overflow: hidden;
  }

  .mini-core-fill {
    width: 100%;
    transition: height 0.3s ease;
  }

  .disks-list {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .disk-item {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.65rem 0.75rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .disk-meta-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
  }

  .disk-name-wrap {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .disk-mount {
    font-weight: 700;
    color: var(--color-text);
  }

  .disk-label {
    color: var(--color-text-muted);
  }

  .fs-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.35rem;
    border-radius: 0.2rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
  }

  .disk-usage-fraction {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-weight: 600;
  }

  .disk-free-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .removable-badge {
    font-size: 0.68rem;
    padding: 0.1rem 0.35rem;
    border-radius: 0.2rem;
    background-color: var(--color-surface);
    color: var(--color-secondary);
  }

  .temp-readings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 0.5rem;
  }

  .temp-reading-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .temp-label {
    font-size: 0.8rem;
    color: var(--color-text);
    font-weight: 500;
  }

  .temp-value-metric {
    font-size: 0.95rem;
    font-weight: 700;
    font-family: monospace;
  }

  .gpu-list,
  .network-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .gpu-item,
  .network-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.6rem 0.75rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .gpu-name-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .gpu-name {
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .vram-tag {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
    background-color: var(--color-surface);
    color: var(--color-primary);
  }

  .gpu-driver-row,
  .net-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
  }

  .driver-num,
  .net-val {
    color: var(--color-text);
  }

  .mono {
    font-family: monospace;
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

  .net-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-text-muted);
  }

  .net-status-dot.active {
    background-color: var(--color-success);
  }

  .transfer-stats {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    gap: 1rem;
    margin-top: 0.15rem;
  }

  .skeleton-container {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 0.85rem;
    flex: 1;
    overflow-y: auto;
    background-color: var(--color-base);
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
