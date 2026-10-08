<script lang="ts">
  import { page } from "$app/state";
  import { slide } from "svelte/transition";

  let expanded = $state(true);

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
    {#if expanded}
      <span>Collapse</span>
    {:else}
      <span>Expand</span>
    {/if}
  </button>
</nav>

<style>
  .navbar {
    --link-margin: 4px;
    --link-padding: 8px;
    --link-hover-padding: calc(var(--link-padding) * 2);

    color: var(--color-text);
  }

  .toggle-expand-button {
    display: block;
    position: relative;
    margin-top: calc(-1 * var(--width-border));
    margin-inline: auto 4vw;
    padding: 8px;

    border: solid var(--width-border) var(--color-border);
    border-top: none;
    border-radius: 0 0 8px 8px;
    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .page-links {
    display: grid;
    margin: 0;
    padding: 0;
    grid-template-columns: repeat(auto-fit, minmax(max(22%, 120px), 1fr));
    max-width: 100%;
    align-items: center;
    justify-content: center;
    border-bottom: solid var(--width-border) var(--color-border);

    background-color: var(--color-surface);
    list-style: none;
  }

  .page-links li {
    display: flex;
    min-height: calc(1lh + (var(--link-margin) + var(--link-hover-padding) + var(--width-border)) * 2);
    align-items: center;
    justify-content: center;

    line-height: var(--link-line-height);
  }

  .page-link {
    display: inline-block;
    margin: var(--link-margin);
    padding: var(--link-padding);
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;

    background-color: var(--color-surface);
    line-height: inherit;
    text-decoration: none;
    transition: all 0.32s ease;
  }

  .page-link:hover {
    padding: var(--link-hover-padding);

    font-weight: 800;
    text-shadow: var(--color-primary) 0 0 16px;
  }
</style>
