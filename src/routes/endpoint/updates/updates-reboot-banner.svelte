<script lang="ts">
  interface Props {
    onRestart: () => void;
  }

  let { onRestart }: Props = $props();
</script>

<div class="reboot-hero-banner" role="alert">
  <div class="reboot-hero-left">
    <div class="reboot-hero-icon">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"
        />
        <line x1="12" y1="9" x2="12" y2="13" />
        <line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
    </div>
    <div class="reboot-hero-text">
      <span class="reboot-hero-title">System Restart Required</span>
      <span class="reboot-hero-desc">
        A restart is pending to complete installed updates. Additional update installations are
        temporarily paused until the system is restarted.
      </span>
    </div>
  </div>
  <button class="btn-hero-restart" onclick={onRestart}>
    Restart Now
  </button>
</div>

<style>
  .reboot-hero-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1rem;
    background: linear-gradient(
      90deg,
      color-mix(in srgb, var(--color-warning) 18%, var(--color-surface)) 0%,
      color-mix(in srgb, var(--color-warning) 10%, var(--color-surface)) 100%
    );
    border: solid var(--width-border) var(--color-warning);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
    gap: 1rem;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .reboot-hero-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 260px;
  }

  .reboot-hero-icon {
    color: var(--color-warning);
    flex-shrink: 0;
  }

  .reboot-hero-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .reboot-hero-title {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .reboot-hero-desc {
    font-size: 0.76rem;
    color: var(--color-text-muted);
    line-height: 1.35;
  }

  .btn-hero-restart {
    background-color: var(--color-warning);
    border: solid 1px var(--color-warning);
    color: var(--color-base);
    font-weight: 700;
    font-size: 0.8rem;
    padding: 0.4rem 0.9rem;
    border-radius: 0.3rem;
    cursor: pointer;
    white-space: nowrap;
    transition: filter 0.15s ease;
  }

  .btn-hero-restart:hover {
    filter: brightness(1.1);
  }
</style>
