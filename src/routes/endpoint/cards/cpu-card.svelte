<script lang="ts">
  import { getTemperatureColor } from "../metrics";
  import ProgressBar from "../progress-bar.svelte";
  import type { CpuInfo } from "../types";

  interface Props {
    cpu: CpuInfo;
  }

  let { cpu }: Props = $props();
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
        <rect width="16" height="16" x="4" y="4" rx="2" />
        <rect width="6" height="6" x="9" y="9" rx="1" />
        <path d="M15 2v2" /><path d="M15 20v2" /><path d="M2 15h2" /><path d="M2 9h2" />
        <path d="M20 15h2" /><path d="M20 9h2" /><path d="M9 2v2" /><path d="M9 20v2" />
      </svg>
      <h3 class="card-title">Processor (CPU)</h3>
    </div>
    {#if cpu.temperature_celsius != null}
      {@const c = cpu.temperature_celsius}
      <span
        class="temp-chip"
        style:color={getTemperatureColor(c)}
        title="CPU Temperature"
      >
        {c.toFixed(1)}°C
      </span>
    {/if}
  </div>

  <div class="item-lead">{cpu.model_name}</div>

  <div class="metric-specs-grid">
    <div class="spec-cell">
      <span class="spec-label">Cores</span>
      <span class="spec-value">{cpu.physical_cores}P / {cpu.logical_cores}T</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Frequency</span>
      <span class="spec-value">{cpu.frequency_mhz} MHz</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Utilization</span>
      <span class="spec-value">{cpu.global_usage_percent.toFixed(1)}%</span>
    </div>
  </div>

  <div class="progress-block">
    <ProgressBar
      value={cpu.global_usage_percent}
      label="Overall CPU Load"
      showLabel={true}
      height="9px"
    />
  </div>

  {#if cpu.core_usages.length > 0}
    <div class="cores-subgrid">
      {#each cpu.core_usages as usage, i}
        <div class="mini-core-cell" title={`Core ${i + 1}: ${usage.toFixed(0)}%`}>
          <span class="core-num">C{i + 1}</span>
          <div class="mini-core-bar">
            <div
              class="mini-core-fill"
              style:height={`${Math.min(100, usage)}%`}
              style:background-color={getTemperatureColor(
                usage < 70 ? 50 : usage < 85 ? 75 : 85,
              )}
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
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

  .temp-chip {
    font-size: 0.85rem;
    font-weight: 700;
    font-family: monospace;
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
</style>
