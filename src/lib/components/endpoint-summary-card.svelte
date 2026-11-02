<script lang="ts">
  import { AlertTriangle, Circle, Cpu } from "$lib/components/icons";
  import { endpointStore } from "../../routes/endpoint/endpoint.svelte";
  import {
    formatBytes,
    formatUptime,
    getTemperatureColor,
    getUsageColor,
  } from "../../routes/endpoint/metrics";
  import { onDestroy, onMount } from "svelte";

  const store = endpointStore;

  onMount(() => {
    store.subscribe().catch((err) => {
      console.warn("Failed to subscribe to endpoint metrics:", err);
    });
  });

  onDestroy(() => {
    store.unsubscribe();
  });

  let primaryDisk = $derived.by(() => {
    if (!store.machine?.disks || store.machine.disks.length === 0) return null;
    return (
      store.machine.disks.find((d) => d.mount_point.toUpperCase().startsWith("C")) ??
      store.machine.disks[0]
    );
  });

  let cpuPercent = $derived(store.machine?.cpu.global_usage_percent ?? 0);
  let memPercent = $derived(store.machine?.memory.usage_percent ?? 0);
  let diskPercent = $derived(primaryDisk?.usage_percent ?? 0);

  let cpuColor = $derived(getUsageColor(cpuPercent));
  let memColor = $derived(getUsageColor(memPercent));
  let diskColor = $derived(getUsageColor(diskPercent));

  let hasHighUsage = $derived(cpuPercent >= 85 || memPercent >= 85 || diskPercent >= 85);
  let hasModerateUsage = $derived(
    !hasHighUsage && (cpuPercent >= 70 || memPercent >= 70 || diskPercent >= 70),
  );
</script>

<a href="/hardware" class="card" aria-label="Hardware system summary">
  <div class="card-header-row">
    <h2 class="card-header">
      <Cpu size={20} />
      <span>Hardware</span>
    </h2>
    {#if store.machine?.system.computer_name}
      <span class="host-badge">{store.machine.system.computer_name}</span>
    {/if}
  </div>

  {#if store.machineError}
    <div class="error-banner">{store.machineError}</div>
  {:else if store.osSupport && !store.osSupport.is_windows}
    <div class="unsupported-banner">
      Unsupported OS ({store.osSupport.os_name})
    </div>
  {:else if store.machineLoading && !store.machine}
    <p class="status-message">Reading system metrics…</p>
  {:else if store.machine}
    <p
      class="headline"
      class:ok={!hasHighUsage && !hasModerateUsage && !store.updates?.reboot_required}
      class:warn={hasModerateUsage || (!hasHighUsage && Boolean(store.updates?.reboot_required))}
      class:crit={hasHighUsage}
    >
      {#if hasHighUsage || hasModerateUsage || store.updates?.reboot_required}
        <AlertTriangle size={16} />
      {/if}
      <span>
        {#if store.updates?.reboot_required}
          Restart required to complete updates
        {:else if hasHighUsage}
          High resource utilization detected
        {:else if hasModerateUsage}
          Elevated resource utilization
        {:else}
          System operating normally
        {/if}
      </span>
    </p>

    <dl class="stats">
      <div class="stat">
        <dt class="stat-label">
          <Circle size={8} class="dot" fill={cpuColor} />
          CPU
        </dt>
        <dd class="stat-value" style:color={cpuColor}>
          {cpuPercent.toFixed(0)}%
        </dd>
        <span class="stat-sub">
          {store.machine.cpu.physical_cores}P / {store.machine.cpu.logical_cores}T
        </span>
      </div>

      <div class="stat">
        <dt class="stat-label">
          <Circle size={8} class="dot" fill={memColor} />
          RAM
        </dt>
        <dd class="stat-value" style:color={memColor}>
          {memPercent.toFixed(0)}%
        </dd>
        <span class="stat-sub">
          {formatBytes(store.machine.memory.used_bytes)} / {formatBytes(store.machine.memory.total_bytes)}
        </span>
      </div>

      {#if primaryDisk}
        <div class="stat">
          <dt class="stat-label">
            <Circle size={8} class="dot" fill={diskColor} />
            {primaryDisk.mount_point || "Disk"}
          </dt>
          <dd class="stat-value" style:color={diskColor}>
            {diskPercent.toFixed(0)}%
          </dd>
          <span class="stat-sub">
            {formatBytes(primaryDisk.available_bytes)} free
          </span>
        </div>
      {/if}

      <div class="stat">
        {#if store.machine.cpu.temperature_celsius != null}
          {@const temp = store.machine.cpu.temperature_celsius}
          {@const tColor = getTemperatureColor(temp)}
          <dt class="stat-label">
            <Circle size={8} class="dot" fill={tColor} />
            Temp
          </dt>
          <dd class="stat-value" style:color={tColor}>
            {temp.toFixed(0)}°C
          </dd>
          <span class="stat-sub">
            Up {formatUptime(store.machine.system.uptime_seconds)}
          </span>
        {:else}
          <dt class="stat-label">Uptime</dt>
          <dd class="stat-value uptime-text">
            {formatUptime(store.machine.system.uptime_seconds)}
          </dd>
          <span class="stat-sub">
            {store.machine.system.architecture}
          </span>
        {/if}
        {#if store.machine.system.fast_startup_enabled}
          <span
            class="fast-startup-warn"
            role="status"
            title="Fast startup is turned on"
          >
            <AlertTriangle size={11} />
            <span>Fast startup is turned on</span>
          </span>
        {/if}
      </div>
    </dl>

    <ul class="detail-list">
      <li class="detail-item">
        <span class="detail-label">OS:</span>
        <span class="detail-value">
          {store.machine.system.os_name}
          {#if store.machine.system.build_number}
            (Build {store.machine.system.build_number})
          {/if}
        </span>
      </li>
      <li class="detail-item">
        <span class="detail-label">Hardware:</span>
        <span class="detail-value truncated" title={store.machine.cpu.model_name}>
          {store.machine.cpu.model_name}
        </span>
      </li>
      {#if store.shell}
        <li class="detail-item">
          <span class="detail-label">Shell:</span>
          <span class="detail-value shell-tag">
            {store.shell.shell_name} {store.shell.version}
          </span>
        </li>
      {/if}
    </ul>
  {/if}
</a>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;
    background-color: var(--color-surface);
    color: var(--color-text);
    text-decoration: none;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .card:hover {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
  }

  .card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .card-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 1.1rem;
    color: var(--color-primary);
  }

  .host-badge {
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .headline {
    margin: 0;
    font-weight: 600;
    color: var(--color-text);
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .headline.ok {
    color: var(--color-success);
  }

  .headline.warn {
    color: var(--color-warning);
  }

  .headline.crit {
    color: var(--color-failure);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
    gap: 8px;
    margin: 0;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .stat-value {
    margin: 0;
    font-size: 1.45rem;
    font-weight: 700;
    font-family: monospace;
    line-height: 1.1;
  }

  .stat-value.uptime-text {
    font-size: 1.1rem;
    color: var(--color-text);
  }

  .stat-sub {
    font-size: 0.72rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .fast-startup-warn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--color-warning);
    line-height: 1.2;
    margin-top: 2px;
  }

  :global(.dot) {
    flex-shrink: 0;
  }

  .detail-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 12px 0 0;
    list-style: none;
    border-top: solid var(--width-border) var(--color-border);
  }

  .detail-item {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: 0.85rem;
  }

  .detail-label {
    color: var(--color-text-muted);
    font-weight: 500;
    flex-shrink: 0;
  }

  .detail-value {
    color: var(--color-text);
  }

  .detail-value.truncated {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shell-tag {
    font-family: monospace;
    font-size: 0.78rem;
  }

  .unsupported-banner {
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    background-color: color-mix(in srgb, var(--color-failure) 15%, transparent);
    color: var(--color-failure);
    border: solid var(--width-border) var(--color-failure);
    font-size: 0.85rem;
    font-weight: 600;
  }
</style>
