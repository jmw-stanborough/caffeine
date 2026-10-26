<script lang="ts">
  import type { MotherboardInfo } from "../types";

  interface Props {
    motherboard: MotherboardInfo;
  }

  let { motherboard }: Props = $props();
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
      <span class="spec-value">
        {motherboard.manufacturer} {motherboard.product}
      </span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">BIOS</span>
      <span class="spec-value">{motherboard.bios_version}</span>
    </div>
    {#if motherboard.bios_date}
      <div class="spec-cell">
        <span class="spec-label">Date</span>
        <span class="spec-value">{motherboard.bios_date}</span>
      </div>
    {/if}
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
</style>
