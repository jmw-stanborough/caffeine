<script lang="ts">
  import TerminalView from "$lib/terminal/terminal-view.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import type { ShellDetectionResult } from "./types";

  interface Props {
    shell: ShellDetectionResult | null;
  }

  let { shell }: Props = $props();

  let terminalView = $state<ReturnType<typeof TerminalView> | null>(null);
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let isConnecting = $state(true);

  async function startTerminalSession() {
    if (!terminalView) return;
    const dims = terminalView.getDimensions();

    try {
      await invoke("endpoint_start_terminal", { cols: dims.cols, rows: dims.rows });
      isConnecting = false;
    } catch (err: unknown) {
      terminalView.writeln(`\r\n\x1b[31mFailed to launch embedded terminal: ${err}\x1b[0m\r\n`);
      isConnecting = false;
      return;
    }

    unlistenData = await listen<string>("terminal://data", (event) => {
      terminalView?.write(event.payload);
    });

    unlistenExit = await listen("terminal://exit", () => {
      terminalView?.writeln("\r\n\x1b[33m[Process completed]\x1b[0m\r\n");
    });
  }

  function handleData(data: string) {
    invoke("endpoint_write_terminal", { data }).catch((err) => {
      console.error("Failed to send terminal input:", err);
    });
  }

  function handleResize(cols: number, rows: number) {
    invoke("endpoint_resize_terminal", { cols, rows }).catch((err) => {
      console.error("Failed to resize terminal:", err);
    });
  }

  function sendData(data: string) {
    handleData(data);
  }

  function handleInterrupt() {
    sendData("\x03");
  }

  function handleClear() {
    terminalView?.clear();
    sendData("Clear-Host\r");
  }

  async function handleRestart() {
    cleanupListeners();
    invoke("endpoint_close_terminal").catch((err) => {
      console.error("Failed to close terminal session:", err);
    });
    terminalView?.clear();
    isConnecting = true;
    await startTerminalSession();
  }

  function cleanupListeners() {
    if (unlistenData) {
      unlistenData();
      unlistenData = null;
    }
    if (unlistenExit) {
      unlistenExit();
      unlistenExit = null;
    }
  }

  onMount(() => {
    startTerminalSession();
  });

  onDestroy(() => {
    cleanupListeners();
    invoke("endpoint_close_terminal").catch((err) => {
      console.error("Failed to close terminal session:", err);
    });
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
          title={`Executable: ${shell.executable_path} (Version: ${shell.version})`}
        >
          {shell.shell_name}
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

      <div class="action-divider"></div>

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

  <div class="terminal-body">
    <TerminalView
      bind:this={terminalView}
      onData={handleData}
      onResize={handleResize}
    />
  </div>
</div>

<style>
  .terminal-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.55rem 0.85rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
    min-width: 0;
  }

  .header-icon-wrap {
    color: var(--color-primary);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .window-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
    white-space: nowrap;
  }

  .shell-badge {
    font-size: 0.72rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    background-color: var(--color-surface-hover);
    color: var(--color-text-muted);
    border: solid 1px var(--color-border);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .shell-badge.pwsh {
    background-color: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
    color: var(--color-primary);
    border-color: color-mix(in srgb, var(--color-primary) 35%, transparent);
  }

  .connecting-label {
    font-size: 0.75rem;
    color: var(--color-secondary);
    font-style: italic;
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin-left: auto;
    flex-shrink: 0;
  }

  .quick-commands {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .btn-quick {
    background-color: var(--color-base);
    border: solid 1px var(--color-border);
    color: var(--color-text);
    padding: 0.18rem 0.45rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-family: monospace;
    cursor: pointer;
    transition: background-color 0.12s ease, border-color 0.12s ease;
  }

  .btn-quick:hover {
    background-color: var(--color-surface-hover);
    border-color: var(--color-primary);
  }

  .action-divider {
    width: 1px;
    height: 14px;
    background-color: var(--color-border);
    margin: 0 0.15rem;
    flex-shrink: 0;
  }

  .btn-interrupt {
    background-color: color-mix(in srgb, var(--color-failure) 15%, var(--color-surface));
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
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
    padding: 0.18rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-action:hover {
    background-color: var(--color-surface-hover);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .terminal-body {
    flex: 1;
    min-height: 0;
    padding: 0.4rem 0.5rem;
    background-color: var(--color-base);
    overflow: hidden;
  }
</style>
