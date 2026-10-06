<script lang="ts">
  import { slide } from "svelte/transition";
  import { page } from "$app/state";

  let expanded = $state(false);

  const pageLinks = [
    { href: "/", label: "Dashboard" },
    { href: "/server-monitor", label: "Server Monitor" },
  ];
</script>

<nav class="navbar">
  {#if expanded}
    <ul class="page-links" transition:slide={{ duration: 500 }}>
      {#each pageLinks as {href, label}}
        {@const isActive = href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href)}
        <li>
          <a class="page-link" href={href} class:active={isActive} aria-current={isActive ? "page" : undefined}>
            {label}
          </a>
        </li>
      {/each}
    </ul>
  {/if}

  <button class="toggle-expand-button" aria-expanded={expanded} aria-controls="nav-menu" onclick={() => (expanded = !expanded)}>
    <span>Navigation</span>
  </button>
</nav>

<style>
  .navbar {
    display: inline-block;

    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .toggle-expand-button {
    padding: 8px;

    border: solid var(--width-border) var(--color-border);
    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .page-links {
    display: flex;
    list-style: none;
  }

  .page-link {
    display: inline-block;
    margin: 4px;
    padding: 8px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;

    background-color: var(--color-surface);
    text-decoration: none;
    transition: all 0.32s ease;
  }

  .page-link:hover {
    padding: 16px;

    font-weight: 800;
  }
</style>
