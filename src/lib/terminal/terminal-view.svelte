<script lang="ts">
  import { FitAddon } from "@xterm/addon-fit";
  import { Terminal } from "@xterm/xterm";
  import "@xterm/xterm/css/xterm.css";
  import { onDestroy, onMount } from "svelte";
  import { getTerminalTheme } from "./theme";

  interface Props {
    onData?: (data: string) => void;
    onResize?: (cols: number, rows: number) => void;
  }

  let { onData, onResize }: Props = $props();

  let containerEl = $state<HTMLDivElement | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let darkModeMql: MediaQueryList | null = null;
  let themeListener: ((e: MediaQueryListEvent) => void) | null = null;
  let themeObserver: MutationObserver | null = null;

  function isDarkThemeActive(): boolean {
    if (typeof document === "undefined") return true;
    const explicit = document.documentElement.dataset.theme;
    if (explicit === "dark") return true;
    if (explicit === "light") return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  export function write(data: string) {
    term?.write(data);
  }

  export function writeln(data: string) {
    term?.writeln(data);
  }

  export function writeError(message: string, err?: unknown) {
    const detail = err !== undefined ? `: ${err}` : "";
    term?.writeln(`\r\n\x1b[31m${message}${detail}\x1b[0m\r\n`);
  }

  export function clear() {
    term?.clear();
  }

  export function focus() {
    term?.focus();
  }

  export function fit() {
    if (fitAddon && containerEl && containerEl.clientWidth > 0 && containerEl.clientHeight > 0) {
      fitAddon.fit();
      if (term && onResize) {
        onResize(term.cols, term.rows);
      }
    }
  }

  export function getDimensions(): { cols: number; rows: number } {
    return {
      cols: term?.cols || 80,
      rows: term?.rows || 24,
    };
  }

  onMount(() => {
    if (!containerEl) return;

    darkModeMql = window.matchMedia("(prefers-color-scheme: dark)");
    const theme = getTerminalTheme(isDarkThemeActive());

    term = new Terminal({
      cursorBlink: true,
      cursorStyle: "bar",
      fontSize: 13,
      lineHeight: 1.25,
      fontFamily: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
      theme,
      convertEol: true,
      allowProposedApi: true,
    });

    themeListener = () => {
      if (term) {
        term.options.theme = getTerminalTheme(isDarkThemeActive());
      }
    };
    darkModeMql.addEventListener("change", themeListener);

    themeObserver = new MutationObserver(() => {
      if (term) {
        term.options.theme = getTerminalTheme(isDarkThemeActive());
      }
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerEl);

    if (containerEl.clientWidth > 0 && containerEl.clientHeight > 0) {
      fitAddon.fit();
    }

    term.onData((data) => {
      onData?.(data);
    });

    resizeObserver = new ResizeObserver(() => {
      if (
        fitAddon &&
        term &&
        containerEl &&
        containerEl.clientWidth > 0 &&
        containerEl.clientHeight > 0
      ) {
        fitAddon.fit();
        onResize?.(term.cols, term.rows);
      }
    });
    resizeObserver.observe(containerEl);
    term.focus();
  });

  onDestroy(() => {
    if (themeObserver) {
      themeObserver.disconnect();
      themeObserver = null;
    }
    if (darkModeMql && themeListener) {
      darkModeMql.removeEventListener("change", themeListener);
      themeListener = null;
      darkModeMql = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    if (term) {
      term.dispose();
      term = null;
    }
  });
</script>

<div class="terminal-canvas-container" bind:this={containerEl}></div>

<style>
  .terminal-canvas-container {
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background-color: var(--color-surface);
  }

  .terminal-canvas-container :global(.xterm) {
    height: 100%;
    padding: 0.5rem;
    box-sizing: border-box;
  }

  .terminal-canvas-container :global(.xterm-viewport) {
    overflow-y: auto;
  }
</style>
