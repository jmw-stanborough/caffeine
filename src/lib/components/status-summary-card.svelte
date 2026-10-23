<script lang="ts">
  import type { MonitorStatus } from "$lib/monitor-store.svelte";
  import type { Component } from "svelte";

  export interface AttentionItem {
    id: string;
    label: string;
    detail: string;
    severity: "offline" | "degraded";
  }

  interface Props {
    title: string;
    href: string;
    noun: string;
    loading: boolean;
    errorMessage: string | null;
    total: number;
    counts: Record<MonitorStatus, number>;
    attention: AttentionItem[];
    icon?: Component<{ size?: number | string; class?: string }>;
    extraStat?: { label: string; value: string };
  }

  let {
    title,
    href,
    noun,
    loading,
    errorMessage,
    total,
    counts,
    attention,
    icon: Icon = undefined,
    extraStat,
  }: Props = $props();

  const STAT_ORDER: { key: MonitorStatus; label: string }[] = [
    { key: "online", label: "Healthy" },
    { key: "degraded", label: "Warning" },
    { key: "offline", label: "Error" },
    { key: "checking", label: "Checking" },
  ];
</script>

<a {href} class="card">
  <h2 class="card-header">
    {#if Icon}
      <Icon size={20} />
    {/if}
    <span>{title}</span>
  </h2>

  {#if errorMessage}
    <div class="error-banner">{errorMessage}</div>
  {:else if loading}
    <p class="status-message">Loading…</p>
  {:else if total === 0}
    <p class="status-message">No {noun}s configured.</p>
  {:else}
    <p class="headline" class:ok={attention.length === 0}>
      {#if attention.length === 0}
        All {total} {noun}{total === 1 ? "" : "s"} healthy
      {:else}
        {attention.length} of {total} {noun}{total === 1 ? "" : "s"} need attention
      {/if}
    </p>

    <dl class="stats">
      {#each STAT_ORDER as { key, label }}
        <div class="stat" class:dim={counts[key] === 0}>
          <dt class="stat-label"><span class="dot dot-{key}"></span>{label}</dt>
          <dd class="stat-value">{counts[key]}</dd>
        </div>
      {/each}
      {#if extraStat}
        <div class="stat">
          <dt class="stat-label">{extraStat.label}</dt>
          <dd class="stat-value">{extraStat.value}</dd>
        </div>
      {/if}
    </dl>

    {#if attention.length > 0}
      <ul class="attention-list">
        {#each attention as item (item.id)}
          <li class="attention-item">
            <span class="dot dot-{item.severity}"></span>
            <span class="attention-label">{item.label}</span>
            <span class="attention-detail">{item.detail}</span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</a>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;
    background-color: var(--color-surface);
    color: var(--color-text);
    text-decoration: none;
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .card:hover {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
  }

  .card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 1.1rem;
    color: var(--color-primary);
  }

  .headline {
    margin: 0;
    font-weight: 600;
    color: var(--color-secondary);
  }

  .headline.ok {
    color: var(--color-primary);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
    gap: 8px;
    margin: 0;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat.dim {
    opacity: 0.5;
  }

  .stat-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .stat-value {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .dot-online {
    background-color: var(--color-success);
  }

  .dot-degraded {
    background-color: var(--color-warning);
  }

  .dot-offline {
    background-color: var(--color-failure);
  }

  .attention-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 12px 0 0;
    list-style: none;
    border-top: solid var(--width-border) var(--color-border);
  }

  .attention-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 0.9rem;
  }

  .attention-label {
    font-weight: 600;
  }

  .attention-detail {
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
