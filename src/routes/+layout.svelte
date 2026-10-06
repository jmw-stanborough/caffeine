<script>
  import Navbar from "$lib/navbar.svelte";

  let { children } = $props();
</script>

<header><Navbar /></header>

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
    --color-primary-h: 135;
    --color-primary: oklch(var(--color-primary-l) var(--color-primary-c) var(--color-primary-h));

    --color-base-l: calc(62 / 64);
    --color-surface-l: calc(54 / 64);
    --color-border-l: calc(46 / 64);
    --color-text-l: calc(16/ 64);
    --color-text-muted-l: calc(32 / 64);
    --color-link-l: calc(32/ 64);
    --color-link-hover-l: calc(40 / 64);
    --color-link-visited-l: calc(24 / 64);

    --color-base: oklch(from var(--color-primary) var(--color-base-l) calc(c / 16) h);
    --color-surface: oklch(from var(--color-primary) var(--color-surface-l) calc(c / 4) h);
    --color-border: oklch(from var(--color-primary) var(--color-border-l) calc(c / 4) h);
    --color-text: oklch(from var(--color-primary) var(--color-text-l) calc(c / 4) h);
    --color-text-muted: oklch(from var(--color-primary) var(--color-text-muted-l) calc(c / 4) h);
    --color-link: oklch(from var(--color-primary) var(--color-link-l) c h);
    --color-link-hover: oklch(from var(--color-primary) var(--color-link-hover-l) c h);
    --color-link-visited: oklch(from var(--color-primary) var(--color-link-visited-l) c h);
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --color-primary-l: calc(48 / 64);
      --color-base-l: calc(8 / 64);
      --color-surface-l: calc(16 / 64);
      --color-border-l: calc(26 / 64);
      --color-text-l: calc(48 / 64);
      --color-text-muted-l: calc(32 / 64);
    }
  }

  :global(body) {
    margin: 0;
    min-height: 100vh;

    background-color: var(--color-base);
    color: var(--color-text);
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
</style>
