<script lang="ts">
  import type { OsSupportInfo } from "./types";

  interface Props {
    osSupport: OsSupportInfo | null;
  }

  let { osSupport }: Props = $props();
</script>

<div class="unsupported-container">
  <div class="warning-card">
    <div class="icon-wrap">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
        <line x1="12" y1="9" x2="12" y2="13" />
        <line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
    </div>

    <h1 class="warning-title">Unsupported Operating System</h1>

    <p class="warning-description">
      The <strong>Endpoint</strong> module provides hardware telemetry, system updates, and
      terminal management and supports <strong>Microsoft Windows</strong>, <strong>Linux</strong>,
      and <strong>macOS</strong>.
    </p>

    <div class="details-box">
      <div class="detail-row">
        <span class="detail-label">Detected Operating System:</span>
        <span class="detail-value">{osSupport?.os_name || "Unknown"}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Architecture:</span>
        <span class="detail-value">{osSupport?.architecture || "Unknown"}</span>
      </div>
      {#if osSupport?.hostname}
        <div class="detail-row">
          <span class="detail-label">Host:</span>
          <span class="detail-value">{osSupport.hostname}</span>
        </div>
      {/if}
      {#if osSupport?.error_message}
        <div class="detail-row error-note">
          <span class="detail-label">Notice:</span>
          <span class="detail-value">{osSupport.error_message}</span>
        </div>
      {/if}
    </div>

    <a href="/" class="btn-home">Return to Dashboard</a>
  </div>
</div>

<style>
  .unsupported-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    min-height: 100%;
    padding: 2rem;
    box-sizing: border-box;
    background-color: var(--color-base);
  }

  .warning-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 540px;
    width: 100%;
    padding: 2.5rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.75rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  }

  .icon-wrap {
    color: var(--color-failure);
    margin-bottom: 1.25rem;
  }

  .warning-title {
    margin: 0 0 1rem 0;
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .warning-description {
    margin: 0 0 1.5rem 0;
    font-size: 0.95rem;
    line-height: 1.5;
    color: var(--color-text);
  }

  .details-box {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
    padding: 1rem 1.25rem;
    margin-bottom: 1.25rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    text-align: left;
    font-size: 0.85rem;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
  }

  .detail-label {
    color: var(--color-text-muted);
  }

  .detail-value {
    color: var(--color-text);
    font-family: monospace;
    font-weight: 600;
  }

  .error-note .detail-value {
    color: var(--color-failure);
    font-family: inherit;
    font-weight: 500;
  }

  .btn-home {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.6rem 1.4rem;
    border-radius: 0.375rem;
    background-color: var(--color-primary);
    color: var(--color-base);
    font-weight: 600;
    font-size: 0.9rem;
    text-decoration: none;
    transition: filter 0.15s ease;
  }

  .btn-home:hover {
    filter: brightness(1.1);
  }
</style>
