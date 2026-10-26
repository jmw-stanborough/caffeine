<script lang="ts">
  import { formatBytes } from "../metrics";
  import ProgressBar from "../progress-bar.svelte";
  import type { DiskInfo } from "../types";

  interface Props {
    disks: DiskInfo[];
  }

  let { disks }: Props = $props();
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
        <line x1="22" x2="2" y1="12" y2="12" />
        <path
          d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"
        />
        <line x1="6" x2="6.01" y1="16" y2="16" />
        <line x1="10" x2="10.01" y1="16" y2="16" />
      </svg>
      <h3 class="card-title">Secondary Storage Devices</h3>
    </div>
    <span class="storage-count-tag">
      {disks.length} {disks.length === 1 ? "drive" : "drives"}
    </span>
  </div>

  <div class="disks-list">
    {#each disks as disk}
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

  .storage-count-tag {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-muted);
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
</style>
