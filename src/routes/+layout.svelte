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

    /* Yes, I have a thing for powers of two*/
    /* Yes, I have a problem */
    /* No, I won't change it */
    --color-primary-l: calc(32 / 64);
    --color-primary-c: calc(16 / 64);
    --color-primary-h: 90;
    --color-primary: oklch(var(--color-primary-l) var(--color-primary-c) var(--color-primary-h));
    --color-secondary: oklch(from var(--color-primary) l c calc(h - 32));

    --color-base-l: calc(62 / 64);
    --color-surface-l: calc(60 / 64);
    --color-border-l: calc(52 / 64);
    --color-text-l: calc(8 / 64);
    --color-text-muted-l: calc(24 / 64);
    --color-link-l: calc(32 / 64);
    --color-link-hover-l: calc(40 / 64);
    --color-link-visited-l: calc(24 / 64);

    --color-base: oklch(from var(--color-primary) var(--color-base-l) calc(c / 32) h);
    --color-surface: oklch(from var(--color-primary) var(--color-surface-l) calc(c / 16) h);
    --color-border: oklch(from var(--color-primary) var(--color-border-l) calc(c / 8) h);
    --color-text: oklch(from var(--color-primary) var(--color-text-l) calc(c / 8) h);
    --color-text-muted: oklch(from var(--color-primary) var(--color-text-muted-l) calc(c / 8) h);
    --color-link: oklch(from var(--color-secondary) var(--color-link-l) c h);
    --color-link-hover: oklch(from var(--color-secondary) var(--color-link-hover-l) c h);
    --color-link-visited: oklch(from var(--color-secondary) var(--color-link-visited-l) c h);
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --color-primary-l: calc(40 / 64);
      --color-base-l: calc(8 / 64);
      --color-surface-l: calc(12 / 64);
      --color-border-l: calc(20 / 64);
      --color-text-l: calc(56 / 64);
      --color-text-muted-l: calc(36 / 64);
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
