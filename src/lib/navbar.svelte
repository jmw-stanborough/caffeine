<script lang="ts">
  import { page } from "$app/state";

  const pageLinks = [
    { href: "/", label: "Dashboard" },
    { href: "/server-monitor", label: "Server Monitor" },
  ];
</script>

<nav>
  <ul class="page-links">
    {#each pageLinks as {href, label}}
      {@const isActive = href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href)}
      <li>
        <a class="page-link" href={href} class:active={isActive} aria-current={isActive ? "page" : undefined}>
          {label}
        </a>
      </li>
    {/each}
  </ul>
</nav>

<style>
  .page-links {
    display: flex;
    list-style: none;
  }

  .page-link {
    display: inline-block;
    margin: 4px;
    padding: 4px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;

    background-color: var(--color-surface);
    text-decoration: none;
    transition: all 0.32s ease;
  }

  .page-link:hover {
    padding: 8px;

    font-weight: 800;
  }
</style>
