<script lang="ts">
  import { page } from "$app/state";

  const navigationItems = [
    { href: "/", label: "Dashboard" },
    { href: "/server-monitor", label: "Server Monitor" }
  ];
</script>

<aside class="sidebar">
  <div class="brand">
    <span class="brand-title">Caffeine</span>
  </div>

  <nav class="nav-list">
    {#each navigationItems as { href, label }}
      {@const isActive = href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href)}
      <a
        class="nav-item"
        class:active={isActive}
        aria-current={isActive ? "page" : undefined}
        {href}
      >
        {label}
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
</style>
