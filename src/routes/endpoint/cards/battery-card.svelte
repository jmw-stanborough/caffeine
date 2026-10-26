<script lang="ts">
  import ProgressBar from "../progress-bar.svelte";
  import type { BatteryInfo } from "../types";

  interface Props {
    battery: BatteryInfo;
  }

  let { battery }: Props = $props();
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
        <rect width="16" height="10" x="2" y="7" rx="2" ry="2" />
        <line x1="22" x2="22" y1="11" y2="13" />
      </svg>
      <h3 class="card-title">Power & Battery</h3>
    </div>
    <span class="battery-status-tag">
      {battery.is_charging ? "Charging" : "Discharging"}
    </span>
  </div>

  <ProgressBar
    value={battery.charge_percent}
    label="Battery Level"
    showLabel={true}
    height="8px"
  />
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

  .battery-status-tag {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }
</style>
