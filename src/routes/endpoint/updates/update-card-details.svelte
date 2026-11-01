<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { SystemUpdateItem } from "../types";

  interface Props {
    update: SystemUpdateItem;
  }

  let { update }: Props = $props();

  let copiedId = $state(false);

  async function handleOpenSupport(url: string) {
    try {
      await openUrl(url);
    } catch (err: unknown) {
      throw new Error(`Failed to open support URL '${url}': ${String(err)}`);
    }
  }

  async function copyUpdateId(id: string) {
    try {
      await navigator.clipboard.writeText(id);
      copiedId = true;
      setTimeout(() => {
        copiedId = false;
      }, 2000);
    } catch (err: unknown) {
      throw new Error(`Failed to copy update ID to clipboard: ${String(err)}`);
    }
  }
</script>

<div class="expanded-details">
  {#if update.description}
    <div class="full-desc">{update.description}</div>
  {/if}

  {#if update.categories.length > 0}
    <div class="categories-row">
      <span class="detail-label">Categories:</span>
      <div class="category-tags">
        {#each update.categories as cat}
          <span class="cat-pill">{cat}</span>
        {/each}
      </div>
    </div>
  {/if}

  <div class="meta-row">
    <span class="detail-label">Update ID:</span>
    <code class="update-id-code">{update.id}</code>
    <button
      class="btn-copy-id"
      onclick={() => copyUpdateId(update.id)}
      title="Copy Update GUID"
    >
      {copiedId ? "Copied!" : "Copy"}
    </button>
  </div>

  {#if update.support_url}
    <div class="meta-row">
      <span class="detail-label">Support Documentation:</span>
      <button
        class="support-link-btn"
        onclick={() => handleOpenSupport(update.support_url!)}
        title={update.support_url}
      >
        Open Microsoft Support Article &rarr;
      </button>
    </div>
  {/if}
</div>

<style>
  .expanded-details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    margin-top: 0.35rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.35rem;
  }

  .full-desc {
    font-size: 0.8rem;
    color: var(--color-text);
    line-height: 1.45;
  }

  .categories-row,
  .meta-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
    font-size: 0.76rem;
  }

  .detail-label {
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .category-tags {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .cat-pill {
    font-size: 0.68rem;
    padding: 0.08rem 0.35rem;
    border-radius: 0.2rem;
    background-color: var(--color-surface);
    color: var(--color-text-muted);
    border: solid var(--width-border) var(--color-border);
  }

  .update-id-code {
    font-size: 0.72rem;
    font-family: monospace;
    background-color: var(--color-surface);
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
  }

  .btn-copy-id {
    background: transparent;
    border: solid 1px var(--color-border);
    border-radius: 0.2rem;
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
    color: var(--color-text);
  }

  .btn-copy-id:hover {
    border-color: var(--color-primary);
    background-color: var(--color-surface);
  }

  .support-link-btn {
    background: transparent;
    border: none;
    color: var(--color-link);
    cursor: pointer;
    font-size: 0.76rem;
    font-weight: 600;
    padding: 0;
    text-decoration: underline;
  }

  .support-link-btn:hover {
    color: var(--color-link-hover);
  }
</style>
