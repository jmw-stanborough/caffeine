<script lang="ts">
  import { getTemperatureColor } from "../metrics";
  import type { TemperatureReading } from "../types";

  interface Props {
    temperatures: TemperatureReading[];
  }

  let { temperatures }: Props = $props();
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
        <path d="M14 4v10.54a4 4 0 1 1-4 0V4a2 2 0 0 1 4 0Z" />
      </svg>
      <h3 class="card-title">Thermal Sensors</h3>
    </div>
  </div>

  <div class="temp-readings-grid">
    {#each temperatures as temp}
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
</style>
