<script>
  import "@fontsource-variable/roboto";
  import Navbar from "$lib/navbar.svelte";

  let { children } = $props();
</script>

<header class="navigation-bar"><Navbar /></header>

<main>
  {@render children()}
</main>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(:root) {
    --width-content: 800px;
    --width-border: 2px;

    --color-primary-l: calc(46 / 64);
    --color-primary-c: calc(15 / 64);
    --color-primary-h: 112; 
    --color-primary: oklch(var(--color-primary-l) var(--color-primary-c) var(--color-primary-h));
    --color-secondary: oklch(from var(--color-primary) l c calc(h - 60));

    --color-base-l: calc(61 / 64);
    --color-surface-l: calc(56 / 64);
    --color-border-l: calc(50 / 64);
    --color-text-l: calc(18 / 64);
    --color-text-muted-l: calc(32 / 64);
    --color-link-l: calc(30 / 64);
    --color-link-hover-l: calc(38 / 64);
    --color-link-visited-l: calc(26 / 64);

    --color-base: oklch(from var(--color-primary) var(--color-base-l) calc(c / 24) h);
    --color-surface: oklch(from var(--color-primary) var(--color-surface-l) calc(c / 16) h);
    --color-border: oklch(from var(--color-primary) var(--color-border-l) calc(c / 8) h);
    --color-text: oklch(from var(--color-primary) var(--color-text-l) calc(c / 8) h);
    --color-text-muted: oklch(from var(--color-primary) var(--color-text-muted-l) calc(c / 8) h);
    --color-link: oklch(from var(--color-secondary) var(--color-link-l) c calc(h + 180));
    --color-link-hover: oklch(from var(--color-secondary) var(--color-link-hover-l) c calc(h + 180));
    --color-link-visited: oklch(from var(--color-secondary) var(--color-link-visited-l) c calc(h + 240));
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --color-primary-l: calc(48 / 64);
      --color-base-l: calc(17.2 / 64);
      --color-surface-l: calc(26.4 / 64);
      --color-border-l: calc(32 / 64);
      --color-text-l: calc(56.5 / 64);
      --color-text-muted-l: calc(42 / 64);
      --color-link-l: calc(46 / 64);
      --color-link-hover-l: calc(54 / 64);
      --color-link-visited-l: calc(40 / 64);
    }
  }

  :global(body) {
    margin: 0;
    min-height: 100vh;

    background-color: var(--color-base);
    color: var(--color-text);
    font-family: "Roboto Variable", sans-serif;
  }

  :global(a) {
    color: var(--color-link);
  }

  :global(a:hover) {
    color: var(--color-link-hover);
  }

  :global(main a:visited) {
    color: var(--color-link-visited);
  }

  .navigation-bar {
    position: sticky;
    top: 0;
  }
</style>
