<script lang="ts">
  import type { SystemMetaInfo } from "../types";

  interface Props {
    system: SystemMetaInfo;
  }

  let { system }: Props = $props();
</script>

<section class="section-card system-card">
  <div class="card-title-row">
    <div class="system-main">
      <span class="host-name">{system.computer_name}</span>
      <span class="domain-tag">{system.domain_or_workgroup}</span>
    </div>
    <span class="arch-badge">{system.architecture}</span>
  </div>
  <div class="os-info-row">
    <span class="os-name">{system.os_name} {system.os_version}</span>
    {#if system.build_number}
      <span class="build-tag">Build {system.build_number}</span>
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

  .system-card {
    background-color: color-mix(in srgb, var(--color-surface) 90%, var(--color-primary) 10%);
    border-color: color-mix(in srgb, var(--color-border) 80%, var(--color-primary) 20%);
  }

  .card-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .system-main {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .host-name {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .domain-tag {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .arch-badge {
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 700;
    color: var(--color-primary);
  }

  .os-info-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .build-tag {
    font-size: 0.75rem;
    padding: 0.1rem 0.4rem;
    border-radius: 0.25rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
  }
</style>
