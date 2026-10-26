<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { FitAddon } from "@xterm/addon-fit";
  import { Terminal } from "@xterm/xterm";
  import "@xterm/xterm/css/xterm.css";
  import { onDestroy, onMount } from "svelte";
  import type { ShellDetectionResult } from "./types";

  interface Props {
    shell: ShellDetectionResult | null;
  }

  let { shell }: Props = $props();

  let terminalContainer = $state<HTMLDivElement | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let isConnecting = $state(true);

  async function initTerminal() {
    if (!terminalContainer) return;

    term = new Terminal({
      cursorBlink: true,
      cursorStyle: "bar",
      fontSize: 13,
      lineHeight: 1.25,
      fontFamily: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
      theme: {
        background: "#1d2021",
        foreground: "#ebdbb2",
        cursor: "#fabd2f",
        cursorAccent: "#1d2021",
        selectionBackground: "#504945",
        black: "#282828",
        red: "#cc241d",
        green: "#98971a",
        yellow: "#d79921",
        blue: "#458588",
        magenta: "#b16286",
        cyan: "#689d6a",
        white: "#a89984",
        brightBlack: "#928374",
        brightRed: "#fb4934",
        brightGreen: "#b8bb26",
        brightYellow: "#fabd2f",
        brightBlue: "#83a598",
        brightMagenta: "#d3869b",
        brightCyan: "#8ec07c",
        brightWhite: "#ebdbb2",
      },
      convertEol: true,
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(terminalContainer);

    if (terminalContainer.clientWidth > 0 && terminalContainer.clientHeight > 0) {
      fitAddon.fit();
    }

    const cols = term.cols || 80;
    const rows = term.rows || 24;

    try {
      await invoke("endpoint_start_terminal", { cols, rows });
      isConnecting = false;
    } catch (err: unknown) {
      term.writeln(`\r\n\x1b[31mFailed to launch embedded terminal: ${err}\x1b[0m\r\n`);
      isConnecting = false;
      return;
    }

    unlistenData = await listen<string>("terminal://data", (event) => {
      term?.write(event.payload);
    });

    unlistenExit = await listen("terminal://exit", () => {
      term?.writeln("\r\n\x1b[33m[Process completed]\x1b[0m\r\n");
    });

    term.onData((data) => {
      invoke("endpoint_write_terminal", { data }).catch((err) => {
        console.error("Failed to send terminal input:", err);
      });
    });

    resizeObserver = new ResizeObserver(() => {
      if (
        fitAddon &&
        term &&
        terminalContainer &&
        terminalContainer.clientWidth > 0 &&
        terminalContainer.clientHeight > 0
      ) {
        fitAddon.fit();
        invoke("endpoint_resize_terminal", {
          cols: term.cols,
          rows: term.rows,
        }).catch((err) => {
          console.error("Failed to resize terminal:", err);
        });
      }
    });

    resizeObserver.observe(terminalContainer);
    term.focus();
  }

  function sendData(data: string) {
    invoke("endpoint_write_terminal", { data }).catch((err) => {
      console.error("Failed to write to terminal:", err);
    });
  }

  function handleInterrupt() {
    sendData("\x03");
  }

  function handleClear() {
    term?.clear();
    sendData("Clear-Host\r");
  }

  async function handleRestart() {
    cleanup();
    if (terminalContainer) {
      terminalContainer.innerHTML = "";
    }
    isConnecting = true;
    await initTerminal();
  }

  function cleanup() {
    if (unlistenData) {
      unlistenData();
      unlistenData = null;
    }
    if (unlistenExit) {
      unlistenExit();
      unlistenExit = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    invoke("endpoint_close_terminal").catch((err) => {
      console.error("Failed to close terminal session:", err);
    });
    if (term) {
      term.dispose();
      term = null;
    }
  }

  onMount(() => {
    initTerminal();
  });

  onDestroy(() => {
    cleanup();
  });
</script>

<div class="terminal-window">
  <div class="terminal-header">
    <div class="header-left">
      <div class="header-icon-wrap">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="4 17 10 11 4 5" />
          <line x1="12" x2="20" y1="19" y2="19" />
        </svg>
      </div>
      <h3 class="window-title">Terminal</h3>
      {#if shell}
        <span
          class="shell-badge"
          class:pwsh={shell.is_pwsh}
          title={`Executable: ${shell.executable_path}`}
        >
          {shell.shell_name} {shell.version}
        </span>
      {/if}
      {#if isConnecting}
        <span class="connecting-label">Connecting PTY...</span>
      {/if}
    </div>

    <div class="header-actions">
      <div class="quick-commands">
        <button
          class="btn-quick"
          onclick={() => sendData("ping 8.8.8.8\r")}
          title="Run ping 8.8.8.8"
        >
          ping
        </button>
        <button
          class="btn-quick"
          onclick={() => sendData("ipconfig\r")}
          title="Run ipconfig"
        >
          ipconfig
        </button>
        <button
          class="btn-quick"
          onclick={() => sendData("Get-Process | Select-Object -First 5\r")}
          title="Show top 5 processes"
        >
          ps
        </button>
      </div>

      <button
        class="btn-interrupt"
        onclick={handleInterrupt}
        title="Send Ctrl+C to cancel running process"
      >
        Ctrl+C
      </button>

      <button
        class="btn-action"
        onclick={handleClear}
        title="Clear terminal screen"
      >
        Clear
      </button>

      <button
        class="btn-action"
        onclick={handleRestart}
        title="Restart terminal session"
      >
        Restart
      </button>
    </div>
  </div>

  <div class="terminal-body" bind:this={terminalContainer}></div>
</div>

<style>
  .terminal-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background-color: #1d2021;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.55rem 0.85rem;
    background-color: #282828;
    border-bottom: solid var(--width-border) #3c3836;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .header-icon-wrap {
    color: var(--color-primary);
    display: flex;
    align-items: center;
  }

  .window-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: #ebdbb2;
  }

  .shell-badge {
    font-size: 0.72rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    background-color: #3c3836;
    color: #a89984;
    border: solid 1px #504945;
  }

  .shell-badge.pwsh {
    background-color: color-mix(in srgb, var(--color-primary) 20%, #282828);
    color: var(--color-primary);
    border-color: var(--color-primary);
  }

  .connecting-label {
    font-size: 0.75rem;
    color: #fabd2f;
    font-style: italic;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .quick-commands {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .btn-quick {
    background-color: #3c3836;
    border: solid 1px #504945;
    color: #ebdbb2;
    padding: 0.18rem 0.45rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-family: monospace;
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .btn-quick:hover {
    background-color: #504945;
  }

  .btn-interrupt {
    background-color: color-mix(in srgb, var(--color-failure) 20%, #282828);
    border: solid 1px var(--color-failure);
    color: var(--color-failure);
    padding: 0.18rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 700;
    font-family: monospace;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-interrupt:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .btn-action {
    background-color: transparent;
    border: solid 1px #504945;
    color: #a89984;
    padding: 0.18rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-action:hover {
    background-color: #3c3836;
    color: #ebdbb2;
  }

  .terminal-body {
    flex: 1;
    min-height: 0;
    padding: 0.4rem 0.5rem;
    background-color: #1d2021;
    overflow: hidden;
  }

  :global(.terminal-body .xterm) {
    height: 100%;
  }

  :global(.terminal-body .xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
