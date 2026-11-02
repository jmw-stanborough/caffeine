<script lang="ts">
  interface Props {
    message: string;
    onRetry: () => void;
  }

  let { message, onRetry }: Props = $props();
</script>

<div class="error-banner">
  <span>{message}</span>
  <button class="btn-retry" onclick={onRetry}>Retry</button>
</div>

<style>
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.55rem 1rem;
    background-color: color-mix(in srgb, var(--color-failure) 14%, var(--color-surface));
    border: solid var(--width-border) var(--color-failure);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
    color: var(--color-failure);
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .btn-retry {
    background: transparent;
    border: solid 1px var(--color-failure);
    color: var(--color-failure);
    font-size: 0.74rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .btn-retry:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }
</style>
