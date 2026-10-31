<script lang="ts">
  import { formatBytes } from "../metrics";

  interface Props {
    totalUpdates: number;
    pendingDownloadCount: number;
    pendingDownloadBytes: number;
    readyToInstallCount: number;
    mandatoryCount: number;
    lastChecked: string;
  }

  let {
    totalUpdates,
    pendingDownloadCount,
    pendingDownloadBytes,
    readyToInstallCount,
    mandatoryCount,
    lastChecked,
  }: Props = $props();
</script>

<div class="stats-overview-bar">
  <div class="stat-cell">
    <span class="stat-value">{totalUpdates}</span>
    <span class="stat-label">Total Updates</span>
  </div>
  <div class="stat-cell">
    <span class="stat-value">{pendingDownloadCount}</span>
    <span class="stat-label">Pending Download ({formatBytes(pendingDownloadBytes)})</span>
  </div>
  <div class="stat-cell">
    <span class="stat-value">{readyToInstallCount}</span>
    <span class="stat-label">Ready to Install</span>
  </div>
  {#if mandatoryCount > 0}
    <div class="stat-cell mandatory">
      <span class="stat-value">{mandatoryCount}</span>
      <span class="stat-label">Mandatory / Security</span>
    </div>
  {/if}
  <div class="stat-cell-meta">
    <span class="stat-label">Last Checked:</span>
    <span class="stat-meta-value">{lastChecked}</span>
  </div>
</div>

<style>
  .stats-overview-bar {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding: 0.6rem 1rem;
    background-color: var(--color-base);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-wrap: wrap;
  }

  .stat-cell {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .stat-value {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .stat-label {
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .stat-cell.mandatory .stat-value {
    color: var(--color-failure);
  }

  .stat-cell-meta {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
  }

  .stat-meta-value {
    font-weight: 600;
    color: var(--color-text);
  }
</style>
