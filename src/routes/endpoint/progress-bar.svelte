<script lang="ts">
  import { getUsageColor } from "./metrics";

  interface Props {
    value: number;
    height?: string;
    showLabel?: boolean;
    label?: string;
  }

  let {
    value = 0,
    height = "8px",
    showLabel = false,
    label = undefined,
  }: Props = $props();

  let clampedValue = $derived(Math.max(0, Math.min(100, Math.round(value))));
  let color = $derived(getUsageColor(clampedValue));
</script>

<div class="progress-wrapper">
  {#if showLabel || label}
    <div class="progress-header">
      {#if label}
        <span class="progress-title">{label}</span>
      {/if}
      {#if showLabel}
        <span class="progress-value" style:color>{clampedValue}%</span>
      {/if}
    </div>
  {/if}
  <div
    class="progress-track"
    style:height
    role="progressbar"
    aria-valuenow={clampedValue}
    aria-valuemin={0}
    aria-valuemax={100}
  >
    <div
      class="progress-fill"
      style:width={`${clampedValue}%`}
      style:background-color={color}
    ></div>
  </div>
</div>

<style>
  .progress-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    width: 100%;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
  }

  .progress-title {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .progress-value {
    font-weight: 700;
    font-family: monospace;
    font-size: 0.85rem;
  }

  .progress-track {
    width: 100%;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.35s ease, background-color 0.35s ease;
  }
</style>
