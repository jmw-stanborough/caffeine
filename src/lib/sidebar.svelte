<script lang="ts">
  import { page } from "$app/state";
  import { Coffee } from "$lib/components/icons";
  import { Cog } from "$lib/icons";
  import SettingsModal from "$lib/components/settings-modal.svelte";
  import {
    DASHBOARD_NAV_ITEM,
    getSidebarSections,
    type SidebarSection,
  } from "$lib/features";

  let sections = $derived<SidebarSection[]>(getSidebarSections());
  let isSettingsOpen = $state(false);
  let isDashboardActive = $derived(page.url.pathname === "/");
</script>

<aside class="sidebar">
  <div class="brand">
    <Coffee size={20} class="brand-icon" />
    <span class="brand-title">Caffeine</span>
  </div>

  <nav class="nav-list">
    <a
      class="nav-item"
      class:active={isDashboardActive}
      aria-current={isDashboardActive ? "page" : undefined}
      href={DASHBOARD_NAV_ITEM.href}
    >
      <DASHBOARD_NAV_ITEM.icon size={16} />
      <span>{DASHBOARD_NAV_ITEM.label}</span>
    </a>

    {#each sections as section, sectionIndex}
      {#if sectionIndex > 0}
        <hr class="sidebar-divider" />
      {/if}
      <div class="sidebar-subheader">
        <section.icon size={16} class="subheader-icon" />
        <span class="subheader-title">{section.label}</span>
      </div>
      {#each section.items as { href, label, badgeCount = 0, icon: Icon }}
        {@const isActive = page.url.pathname.startsWith(href)}
        <a
          class="nav-item is-child"
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
    {/each}
  </nav>

  <div class="sidebar-footer">
    <hr class="sidebar-divider" />
    <button
      type="button"
      class="nav-item settings-button"
      onclick={() => (isSettingsOpen = true)}
      aria-label="Open settings"
    >
      <Cog size={16} />
      <span>Settings</span>
    </button>
  </div>
</aside>

<SettingsModal bind:open={isSettingsOpen} />

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
    justify-content: center;
    gap: 8px;
    height: 48px;
    padding: 0;
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
    flex: 1;
  }

  .sidebar-subheader {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 8px 12px 4px 12px;
    color: var(--color-secondary);
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    user-select: none;
    cursor: default;
  }

  :global(.subheader-icon) {
    color: var(--color-secondary);
    flex-shrink: 0;
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    padding: 8px;
    margin-top: auto;
  }

  .sidebar-divider {
    border: none;
    border-top: solid var(--width-border) var(--color-border);
    margin: 8px 0 4px 0;
  }

  .sidebar-footer .sidebar-divider {
    margin: 0 0 8px 0;
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

  .nav-item.is-child {
    padding-left: 24px;
  }

  .nav-item:hover {
    background-color: var(--color-surface-hover);
  }

  .nav-item.active {
    background-color: var(--color-surface-active);
    color: var(--color-primary);
    font-weight: 600;
  }

  .nav-item.settings-button {
    background: none;
    border: none;
    cursor: pointer;
    width: 100%;
    justify-content: center;
    padding: 8px 0;
    font-family: inherit;
    color: var(--color-link);
  }

  .nav-item.settings-button:hover {
    color: var(--color-link-hover);
  }

  .notification-badge {
    background-color: var(--color-secondary);
    color: var(--color-base);
  }
</style>
