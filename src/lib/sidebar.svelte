<script lang="ts">
  import { page } from "$app/state";
  import { Coffee, LayoutDashboard, Printer, Server } from "$lib/components/icons";
  import { printerStore } from "../routes/printers/printers.svelte";
  import { serverStore } from "../routes/servers/servers.svelte";
  import type { Component } from "svelte";

  interface NavigationItem {
    href: string;
    label: string;
    badgeCount?: number;
    icon: Component<{ size?: number | string; class?: string }>;
  }

  let navigationItems = $derived<NavigationItem[]>([
    {
      href: "/",
      label: "Dashboard",
      icon: LayoutDashboard,
    },
    {
      href: "/printers",
      label: "Printers",
      badgeCount: printerStore.alertCount,
      icon: Printer,
    },
    {
      href: "/servers",
      label: "Servers",
      badgeCount: serverStore.alertCount,
      icon: Server,
    },
  ]);
</script>

<aside class="sidebar">
  <div class="brand">
    <Coffee size={20} class="brand-icon" />
    <span class="brand-title">Caffeine</span>
  </div>

  <nav class="nav-list">
    {#each navigationItems as { href, label, badgeCount = 0, icon: Icon }}
      {@const isActive = href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href)}
      <a
        class="nav-item"
        class:active={isActive}
        aria-current={isActive ? "page" : undefined}
        {href}
      >
        <Icon size={16} />
        <span>{label}</span>
        {#if badgeCount > 0}
          <span
            class="count-badge notification-badge"
            aria-label="{badgeCount} {badgeCount === 1 ? 'alert' : 'alerts'}"
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
    width: max-content;
    height: 100%;
    flex-shrink: 0;
    background-color: var(--color-surface);
    border-right: solid var(--width-border) var(--color-border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 48px;
    padding-inline: 16px;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  :global(.brand-icon) {
    color: var(--color-primary);
    flex-shrink: 0;
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
    gap: 0.5rem;
    padding: 8px 32px 8px 12px;
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

  .notification-badge {
    background-color: var(--color-secondary);
    color: var(--color-base);
  }
</style>
