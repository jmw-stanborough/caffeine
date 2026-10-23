<script lang="ts">
  import { page } from "$app/state";
  import { printerStore } from "../routes/printers/printers.svelte";
  import { serverStore } from "../routes/servers/servers.svelte";

  interface NavigationItem {
    href: string;
    label: string;
    badgeCount?: number;
    hasError?: boolean;
    hasWarning?: boolean;
  }

  let navigationItems = $derived<NavigationItem[]>([
    { href: "/", label: "Dashboard" },
    {
      href: "/servers",
      label: "Servers",
      badgeCount: serverStore.alertCount,
      hasError: serverStore.hasError,
      hasWarning: serverStore.hasWarning,
    },
    {
      href: "/printers",
      label: "Printers",
      badgeCount: printerStore.alertCount,
      hasError: printerStore.hasError,
      hasWarning: printerStore.hasWarning,
    },
  ]);
</script>

<aside class="sidebar">
  <div class="brand">
    <span class="brand-title">Caffeine</span>
  </div>

  <nav class="nav-list">
    {#each navigationItems as { href, label, badgeCount = 0, hasError = false, hasWarning = false }}
      {@const isActive = href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href)}
      <a
        class="nav-item"
        class:active={isActive}
        aria-current={isActive ? "page" : undefined}
        {href}
      >
        <span class="nav-label">{label}</span>
        {#if badgeCount > 0}
          <span
            class="notification-badge"
            class:error={hasError}
            class:warning={!hasError && hasWarning}
            aria-label="{badgeCount} {hasError ? 'errors' : 'warnings'}"
          >
            {badgeCount}
          </span>
        {/if}
      </a>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--width-sidebar);
    height: 100%;
    flex-shrink: 0;
    background-color: var(--color-surface);
    border-right: solid var(--width-border) var(--color-border);
  }

  .brand {
    display: flex;
    align-items: center;
    height: 48px;
    padding-inline: 16px;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .brand-title {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--color-primary);
    letter-spacing: 0.5px;
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    color: var(--color-text);
    font-size: 0.95rem;
    transition: background-color 0.12s ease, color 0.12s ease;
  }

  .nav-item:hover {
    background-color: var(--color-surface-hover);
  }

  .nav-item.active {
    background-color: var(--color-surface-active);
    color: var(--color-primary);
    font-weight: 600;
  }

  .nav-label {
    flex: 1;
    min-width: 0;
  }

  .notification-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.35rem;
    border-radius: 999px;
    font-size: 0.72rem;
    font-weight: 700;
    line-height: 1;
    color: #1c1917;
    background-color: var(--color-warning);
    box-sizing: border-box;
    margin-left: auto;
    flex-shrink: 0;
  }

  .notification-badge.error {
    background-color: var(--color-failure);
  }

  .notification-badge.warning {
    background-color: var(--color-warning);
  }
</style>
