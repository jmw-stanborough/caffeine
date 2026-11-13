<script lang="ts">
  import "./highlight.css";
  import { highlightCode } from "./highlight";

  interface Props {
    value: string;
    languageId: string;
    placeholder?: string;
    onchange?: () => void;
  }

  let {
    value = $bindable(""),
    languageId,
    placeholder = "Write script code here...",
    onchange,
  }: Props = $props();

  let textareaEl = $state<HTMLTextAreaElement | null>(null);
  let overlayEl = $state<HTMLPreElement | null>(null);
  let gutterEl = $state<HTMLDivElement | null>(null);

  let lineCount = $derived(Math.max(1, value.split("\n").length));
  let lineNumbers = $derived(Array.from({ length: lineCount }, (_, idx) => idx + 1));

  let highlightedHtml = $derived.by(() => {
    const rendered = highlightCode(value, languageId);
    // Keep trailing empty line visible in <pre> so scroll height matches <textarea>.
    return value.endsWith("\n") ? `${rendered} ` : rendered;
  });

  function syncScroll() {
    if (!textareaEl) return;
    if (overlayEl) {
      overlayEl.scrollTop = textareaEl.scrollTop;
      overlayEl.scrollLeft = textareaEl.scrollLeft;
    }
    if (gutterEl) {
      gutterEl.scrollTop = textareaEl.scrollTop;
    }
  }

  function handleInput() {
    onchange?.();
    syncScroll();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Tab" || !textareaEl) return;
    event.preventDefault();

    const start = textareaEl.selectionStart;
    const end = textareaEl.selectionEnd;
    const indent = "  ";

    if (event.shiftKey) {
      const lineStart = value.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
      if (value.slice(lineStart).startsWith(indent)) {
        value = value.slice(0, lineStart) + value.slice(lineStart + indent.length);
        onchange?.();
        const nextStart = Math.max(lineStart, start - indent.length);
        const nextEnd = Math.max(lineStart, end - indent.length);
        requestAnimationFrame(() => {
          textareaEl?.setSelectionRange(nextStart, nextEnd);
          syncScroll();
        });
      }
      return;
    }

    value = `${value.slice(0, start)}${indent}${value.slice(end)}`;
    onchange?.();
    const nextCursor = start + indent.length;
    requestAnimationFrame(() => {
      textareaEl?.setSelectionRange(nextCursor, nextCursor);
      syncScroll();
    });
  }
</script>

<div class="code-editor-shell">
  <div class="line-gutter" bind:this={gutterEl} aria-hidden="true">
    {#each lineNumbers as num (num)}
      <div class="line-number">{num}</div>
    {/each}
  </div>

  <div class="editor-stage">
    <pre class="code-overlay" bind:this={overlayEl} aria-hidden="true"><code>{@html highlightedHtml}</code></pre>
    <textarea
      bind:this={textareaEl}
      bind:value
      class="code-input"
      {placeholder}
      aria-label="Script source code"
      spellcheck="false"
      autocomplete="off"
      autocapitalize="off"
      oninput={handleInput}
      onscroll={syncScroll}
      onkeydown={handleKeydown}
    ></textarea>
  </div>
</div>

<style>
  .code-editor-shell {
    flex: 1;
    min-height: 0;
    display: flex;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .line-gutter {
    width: 3.2rem;
    flex-shrink: 0;
    padding: 0.85rem 0.5rem;
    background-color: var(--color-surface);
    border-right: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.88rem;
    line-height: 1.55;
    text-align: right;
    overflow: hidden;
    user-select: none;
  }

  .line-number {
    height: calc(0.88rem * 1.55);
  }

  .editor-stage {
    position: relative;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .code-overlay,
  .code-input {
    position: absolute;
    inset: 0;
    margin: 0;
    padding: 0.85rem 1rem;
    border: none;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.88rem;
    line-height: 1.55;
    letter-spacing: normal;
    tab-size: 2;
    white-space: pre;
    overflow-wrap: normal;
    word-break: normal;
    overflow: auto;
  }

  .code-overlay {
    pointer-events: none;
    color: var(--color-text);
    background-color: transparent;
  }

  .code-overlay code {
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
  }

  .code-input {
    width: 100%;
    height: 100%;
    resize: none;
    outline: none;
    background: transparent;
    color: transparent;
    caret-color: var(--color-text);
  }

  .code-input::placeholder {
    color: var(--color-text-muted);
  }

  .code-input::selection {
    background-color: color-mix(in srgb, var(--color-primary) 28%, transparent);
    color: transparent;
  }
</style>
