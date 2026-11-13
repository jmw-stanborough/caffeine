<script lang="ts">
  import "../scripts/highlight.css";
  import { renderMarkdownToHtml } from "./markdown";

  interface Props {
    content: string;
  }

  let { content }: Props = $props();

  let renderedHtml = $derived(renderMarkdownToHtml(content));
</script>

<div class="markdown-viewer selectable">
  {#if renderedHtml}
    <div class="markdown-body">
      {@html renderedHtml}
    </div>
  {:else}
    <p class="empty-preview">Nothing to preview yet. Start writing Markdown.</p>
  {/if}
</div>

<style>
  .markdown-viewer {
    height: 100%;
    overflow-y: auto;
    padding: 1rem 1.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    line-height: 1.6;
  }

  .empty-preview {
    margin: 0;
    color: var(--color-text-muted);
    font-style: italic;
    font-size: 0.9rem;
  }

  .markdown-body :global(h1),
  .markdown-body :global(h2),
  .markdown-body :global(h3),
  .markdown-body :global(h4),
  .markdown-body :global(h5),
  .markdown-body :global(h6) {
    margin: 1.1em 0 0.45em;
    font-weight: 700;
    line-height: 1.25;
    color: var(--color-text);
  }

  .markdown-body :global(h1:first-child),
  .markdown-body :global(h2:first-child),
  .markdown-body :global(h3:first-child) {
    margin-top: 0;
  }

  .markdown-body :global(h1) {
    font-size: 1.6rem;
    padding-bottom: 0.3rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .markdown-body :global(h2) {
    font-size: 1.3rem;
    padding-bottom: 0.25rem;
    border-bottom: solid var(--width-border) var(--color-border);
  }

  .markdown-body :global(h3) {
    font-size: 1.1rem;
  }

  .markdown-body :global(p) {
    margin: 0.65em 0;
  }

  .markdown-body :global(a) {
    color: var(--color-link);
    text-decoration: underline;
  }

  .markdown-body :global(a:hover) {
    color: var(--color-link-hover);
  }

  .markdown-body :global(code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.88em;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    padding: 0.1rem 0.32rem;
  }

  .markdown-body :global(.md-code-block) {
    margin: 0.85em 0;
    padding: 0.85rem 1rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    overflow-x: auto;
    font-size: 0.86rem;
    line-height: 1.5;
  }

  .markdown-body :global(.md-code-block code) {
    background: transparent;
    border: none;
    padding: 0;
    font-size: inherit;
  }

  .markdown-body :global(blockquote) {
    margin: 0.75em 0;
    padding: 0.25rem 0 0.25rem 0.9rem;
    border-left: 3px solid var(--color-primary);
    color: var(--color-text-muted);
    background-color: color-mix(in srgb, var(--color-surface) 45%, transparent);
    border-radius: 0 0.25rem 0.25rem 0;
  }

  .markdown-body :global(ul),
  .markdown-body :global(ol) {
    margin: 0.6em 0;
    padding-left: 1.4rem;
  }

  .markdown-body :global(ul.task-list) {
    list-style: none;
    padding-left: 0.25rem;
  }

  .markdown-body :global(li.task-list-item) {
    display: flex;
    align-items: flex-start;
    gap: 0.45rem;
    margin: 0.25rem 0;
  }

  .markdown-body :global(li.task-list-item input[type="checkbox"]) {
    margin-top: 0.32rem;
    accent-color: var(--color-primary);
  }

  .markdown-body :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 0.85em 0;
    font-size: 0.9rem;
  }

  .markdown-body :global(th),
  .markdown-body :global(td) {
    border: solid var(--width-border) var(--color-border);
    padding: 0.45rem 0.65rem;
  }

  .markdown-body :global(th) {
    background-color: var(--color-surface);
    font-weight: 700;
  }

  .markdown-body :global(hr) {
    border: none;
    border-top: solid var(--width-border) var(--color-border);
    margin: 1.1rem 0;
  }
</style>
