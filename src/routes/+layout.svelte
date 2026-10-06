<script>
  let { children } = $props();
</script>

<main>
  {@render children()}
</main>

<style>
  *,
  *::before,
  *::after {
    box-sizing: border-box;
  }

  :global(:root) {
    --width-content: 800px;

    /* Yes, lightness and chroma values are in fractions of the powers of two */
    /* Yes, I have a problem */
    /* No, I won't change it */
    --color-primary-l: calc(1 / 2);
    --color-primary-c: calc(1 / 4);
    --color-primary-h: 135;
    --color-primary: oklch(var(--color-primary-l) var(--color-primary-c) var(--color-primary-h));

    --color-base-l: calc(62 / 64);
    --color-surface-l: calc(54 / 64);
    --color-text-l: calc(2 / 64);
    --color-text-muted-l: calc(16 / 64);

    --color-base: oklch(from var(--color-primary) var(--color-base-l) calc(c / 16) h);
    --color-surface: oklch(from var(--color-primary) var(--color-surface-l) calc(c / 4) h);
    --color-text: oklch(from var(--color-primary) var(--color-text-l) calc(c / 8) h);
    --color-text-muted: oklch(from var(--color-primary) var(--color-text-muted-l) calc(c / 4) h);
  }

  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --color-primary-l: calc(12 / 16);
      --color-base-l: calc(1 / 8);
      --color-surface-l: calc(2 / 8);
      --color-text-l: calc(6 / 8);
      --color-text-muted-l: calc(1 / 2);
    }
  }

  :global(body) {
    margin: 0;
    min-height: 100vh;

    background-color: var(--color-base);
  }
</style>
