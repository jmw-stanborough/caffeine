<script lang="ts">
  import { formatBytes } from "../metrics";
  import ProgressBar from "../progress-bar.svelte";
  import type { MemoryInfo } from "../types";

  interface Props {
    memory: MemoryInfo;
  }

  let { memory }: Props = $props();
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
        <path d="M6 19v-3" /><path d="M10 19v-3" /><path d="M14 19v-3" /><path d="M18 19v-3" />
        <rect x="2" y="5" width="20" height="10" rx="2" />
      </svg>
      <h3 class="card-title">Memory (RAM)</h3>
    </div>
    <span class="fraction-label">
      {formatBytes(memory.used_bytes)} / {formatBytes(memory.total_bytes)}
    </span>
  </div>

  <div class="progress-block">
    <ProgressBar
      value={memory.usage_percent}
      label="Memory Allocation"
      showLabel={true}
      height="10px"
    />
  </div>

  <div class="metric-specs-grid">
    <div class="spec-cell">
      <span class="spec-label">Total</span>
      <span class="spec-value">{formatBytes(memory.total_bytes)}</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">In Use</span>
      <span class="spec-value">{formatBytes(memory.used_bytes)}</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Available</span>
      <span class="spec-value">{formatBytes(memory.available_bytes)}</span>
    </div>
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

  .fraction-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
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
</style>
