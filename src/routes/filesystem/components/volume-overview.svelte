<script lang="ts">
  import { HardDrive } from "$lib/components/icons";
  import { formatBytes, getUsageColor } from "../../endpoint/metrics";
  import type { StorageVolume } from "../types";

  interface Props {
    volumes: StorageVolume[];
    selectedVolume: StorageVolume | null;
    onSelectVolume: (vol: StorageVolume) => void;
  }

  let { volumes, selectedVolume, onSelectVolume }: Props = $props();
</script>

<section class="volumes-section" aria-label="Storage Volumes">
  <div class="volumes-grid">
    {#each volumes as volume (volume.id)}
      {@const isSelected = selectedVolume?.id === volume.id}
      {@const usagePercent = Math.round(volume.usage_percent)}
      {@const usageColor = getUsageColor(usagePercent)}

      <button
        type="button"
        class="volume-card"
        class:selected={isSelected}
        onclick={() => onSelectVolume(volume)}
      >
        <div class="card-header">
          <div class="header-main">
            <HardDrive size={18} class="vol-icon" />
            <div class="name-wrap">
              <span class="vol-name">{volume.name}</span>
              <span class="vol-mount">{volume.mount_point}</span>
            </div>
          </div>
          {#if volume.file_system}
            <span class="fs-tag">{volume.file_system}</span>
          {/if}
        </div>

        <div class="usage-stats-row">
          <span class="usage-label">
            {formatBytes(volume.used_bytes)} used of {formatBytes(volume.total_bytes)}
          </span>
          <span class="usage-val" style:color={usageColor}>{usagePercent}%</span>
        </div>

        <div
          class="progress-track"
          role="progressbar"
          aria-valuenow={usagePercent}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            class="progress-fill"
            style:width={`${usagePercent}%`}
            style:background-color={usageColor}
          ></div>
        </div>

        <div class="free-space-row">
          <span class="free-text">{formatBytes(volume.available_bytes)} remaining</span>
          {#if volume.is_removable}
            <span class="badge-removable">Removable</span>
          {/if}
        </div>
      </button>
    {/each}
  </div>
</section>

<style>
  .volumes-section {
    width: 100%;
    margin-bottom: 0.75rem;
  }

  .volumes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.75rem;
  }

  .volume-card {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease;
    color: inherit;
    font: inherit;
  }

  .volume-card:hover {
    border-color: var(--color-border-hover, var(--color-primary));
  }

  .volume-card.selected {
    border-color: var(--color-primary);
    background-color: var(--color-surface-hover, rgba(255, 255, 255, 0.03));
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header-main {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .name-wrap {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .vol-name {
    font-size: 0.875rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .vol-mount {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .fs-tag {
    font-size: 0.6875rem;
    padding: 0.125rem 0.375rem;
    background-color: var(--color-background);
    border-radius: 0.25rem;
    color: var(--color-text-secondary);
    text-transform: uppercase;
  }

  .usage-stats-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
  }

  .usage-label {
    color: var(--color-text-secondary);
  }

  .usage-val {
    font-weight: 600;
  }

  .progress-track {
    width: 100%;
    height: 6px;
    background-color: var(--color-background);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .free-space-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.6875rem;
    color: var(--color-text-secondary);
  }

  .badge-removable {
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background-color: rgba(234, 179, 8, 0.15);
    color: #eab308;
  }
</style>
