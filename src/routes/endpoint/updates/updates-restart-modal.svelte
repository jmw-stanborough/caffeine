<script lang="ts">
  import Modal from "$lib/components/modal.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
    onConfirm: () => void;
  }

  let { open, onClose, onConfirm }: Props = $props();
</script>

<Modal {open} onBackdropClose={onClose}>
  <div class="restart-modal-content">
    <div class="modal-header-row">
      <div class="modal-icon-wrap warning">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
        </svg>
      </div>
      <div>
        <h3 class="modal-title">Restart System</h3>
        <p class="modal-subtitle">A system restart is pending to complete Windows updates.</p>
      </div>
    </div>

    <div class="modal-body-text">
      <p>
        Your computer will restart immediately. Please ensure all your work and open files in other
        applications are saved before continuing.
      </p>
      <p class="modal-note">
        After the computer reboots, Caffeine will automatically restore your monitoring session.
      </p>
    </div>

    <div class="modal-footer-actions">
      <button class="btn-modal-cancel" onclick={onClose}>
        Cancel
      </button>
      <button class="btn-modal-confirm-restart" onclick={onConfirm}>
        Restart Now
      </button>
    </div>
  </div>
</Modal>

<style>
  .restart-modal-content {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    min-width: 380px;
    max-width: 460px;
  }

  .modal-header-row {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
  }

  .modal-icon-wrap {
    padding: 0.6rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-icon-wrap.warning {
    background-color: color-mix(in srgb, var(--color-warning) 18%, var(--color-surface));
    color: var(--color-warning);
  }

  .modal-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
  }

  .modal-subtitle {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .modal-body-text {
    font-size: 0.85rem;
    color: var(--color-text);
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .modal-body-text p {
    margin: 0;
  }

  .modal-note {
    font-size: 0.78rem;
    color: var(--color-text-muted);
    font-style: italic;
  }

  .modal-footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    padding-top: 0.5rem;
  }

  .btn-modal-cancel {
    padding: 0.45rem 0.9rem;
    border-radius: 0.35rem;
    font-size: 0.82rem;
    font-weight: 600;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-modal-cancel:hover {
    border-color: var(--color-text-muted);
  }

  .btn-modal-confirm-restart {
    padding: 0.45rem 1rem;
    border-radius: 0.35rem;
    font-size: 0.82rem;
    font-weight: 700;
    background-color: var(--color-warning);
    border: solid 1px var(--color-warning);
    color: var(--color-base);
    cursor: pointer;
    transition: filter 0.15s ease;
  }

  .btn-modal-confirm-restart:hover {
    filter: brightness(1.1);
  }
</style>
