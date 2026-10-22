<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    maxWidth = "36rem",
    statusVariant = undefined,
    ariaLabel = undefined,
    onBackdropClose = undefined,
    children,
  }: {
    open: boolean;
    maxWidth?: string;
    statusVariant?: "online" | "offline" | "degraded" | "checking";
    ariaLabel?: string;
    onBackdropClose?: () => void;
    children: Snippet;
  } = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      if (onBackdropClose) {
        onBackdropClose();
      } else {
        open = false;
      }
    }
  }

  function handleBackdropKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
    }
  }
</script>

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeyDown}
    role="presentation"
  >
    <div
      class="modal-card {statusVariant ?? ''}"
      style:max-width={maxWidth}
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-label={ariaLabel}
    >
      {@render children()}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    box-sizing: border-box;
    z-index: 100;
  }

  .modal-card {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--status-color, var(--color-border));
    border-radius: 0.5rem;
    width: 100%;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    outline: none;
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
  }

  .modal-card.online {
    --status-color: var(--color-success);
  }

  .modal-card.degraded {
    --status-color: var(--color-warning);
  }

  .modal-card.offline {
    --status-color: var(--color-failure);
  }

  .modal-card.checking {
    --status-color: var(--color-text-muted);
  }
</style>
