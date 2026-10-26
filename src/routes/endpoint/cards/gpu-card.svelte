<script lang="ts">
  import { formatBytes } from "../metrics";
  import type { GpuInfo } from "../types";

  interface Props {
    gpus: GpuInfo[];
  }

  let { gpus }: Props = $props();
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
        <rect width="20" height="14" x="2" y="3" rx="2" />
        <line x1="8" x2="16" y1="21" y2="21" />
        <line x1="12" x2="12" y1="17" y2="21" />
      </svg>
      <h3 class="card-title">Graphics (GPU)</h3>
    </div>
  </div>

  <div class="gpu-list">
    {#each gpus as gpu}
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

  .gpu-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .gpu-item {
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

  .gpu-driver-row {
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

  .driver-num {
    color: var(--color-text);
  }
</style>
