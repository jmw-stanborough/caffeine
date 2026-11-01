<script lang="ts">
  interface Props {
    state: "loading" | "clean" | "no-match" | "no-data";
    lastChecked?: string;
    onCheck: () => void;
    onClearFilters?: () => void;
  }

  let { state, lastChecked, onCheck, onClearFilters }: Props = $props();
</script>

{#if state === "loading"}
  <div class="state-container">
    <div class="spinner"></div>
    <span>Searching Windows Update catalog...</span>
  </div>
{:else if state === "clean"}
  <div class="state-container">
    <div class="clean-icon-wrap">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="42"
        height="42"
        viewBox="0 0 24 24"
        fill="none"
        stroke="var(--color-success)"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" />
        <path d="m9 12 2 2 4-4" />
      </svg>
    </div>
    <span class="clean-title">System is Up to Date</span>
    <span class="clean-sub">No new updates are currently available.</span>
    {#if lastChecked}
      <span class="clean-meta">Last checked: {lastChecked}</span>
    {/if}
    <button class="btn-check-again" onclick={onCheck}>
      Check Again
    </button>
  </div>
{:else if state === "no-match"}
  <div class="state-container">
    <span>No updates match the selected filter or search criteria.</span>
    {#if onClearFilters}
      <button class="btn-clear-search-link" onclick={onClearFilters}>
        Clear filters
      </button>
    {/if}
  </div>
{:else}
  <div class="state-container">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="36"
      height="36"
      viewBox="0 0 24 24"
      fill="none"
      stroke="var(--color-text-muted)"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    <span>No update data loaded.</span>
    <button class="btn-action-primary" onclick={onCheck}>
      Check for Updates Now
    </button>
  </div>
{/if}

<style>
  .state-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1.5rem;
    gap: 0.75rem;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    margin: auto 0;
  }

  .clean-title {
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .clean-sub {
    font-size: 0.82rem;
  }

  .clean-meta {
    font-size: 0.74rem;
    opacity: 0.75;
  }

  .btn-check-again {
    margin-top: 0.5rem;
    padding: 0.38rem 0.85rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-check-again:hover {
    border-color: var(--color-primary);
  }

  .btn-clear-search-link {
    background: transparent;
    border: none;
    color: var(--color-link);
    text-decoration: underline;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .btn-action-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.4rem 0.85rem;
    border-radius: 0.3rem;
    font-size: 0.8rem;
    font-weight: 600;
    background-color: var(--color-primary);
    border: solid 1px var(--color-primary);
    color: var(--color-base);
    cursor: pointer;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
