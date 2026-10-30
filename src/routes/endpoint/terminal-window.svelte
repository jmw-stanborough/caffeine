<script lang="ts">
  import TerminalView from "$lib/terminal/terminal-view.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import type { ShellDetectionResult } from "./types";

  interface Props {
    shell: ShellDetectionResult | null;
    active?: boolean;
  }

  let { shell, active = true }: Props = $props();

  let localShell = $state<ShellDetectionResult | null>(null);
  let effectiveShell = $derived(shell ?? localShell);

  let terminalView = $state<ReturnType<typeof TerminalView> | null>(null);
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let isConnecting = $state(true);

  $effect(() => {
    if (active && terminalView) {
      requestAnimationFrame(() => {
        terminalView?.fit();
        terminalView?.focus();
      });
    }
  });

  async function startTerminalSession() {
    if (!terminalView) return;
    cleanupListeners();
    const dims = terminalView.getDimensions();

    try {
      const detected = await invoke<ShellDetectionResult>("endpoint_start_terminal", {
        cols: dims.cols,
        rows: dims.rows,
      });
      if (detected) {
        localShell = detected;
      }
      isConnecting = false;
    } catch (err: unknown) {
      terminalView?.writeError("Failed to launch embedded terminal", err);
      isConnecting = false;
      return;
    }

    const dataUnlisten = await listen<string>("terminal://data", (event) => {
      terminalView?.write(event.payload);
    });

    const exitUnlisten = await listen("terminal://exit", () => {
      terminalView?.writeln("\r\n\x1b[33m[Process completed]\x1b[0m\r\n");
    });

    unlistenData = dataUnlisten;
    unlistenExit = exitUnlisten;
  }

  function handleData(data: string) {
    invoke("endpoint_write_terminal", { data }).catch((err: unknown) => {
      terminalView?.writeError("Failed to send terminal input", err);
    });
  }

  function handleResize(cols: number, rows: number) {
    invoke("endpoint_resize_terminal", { cols, rows }).catch((err: unknown) => {
      terminalView?.writeError("Failed to resize terminal", err);
    });
  }

  async function handleRestart() {
    cleanupListeners();
    await invoke("endpoint_close_terminal");
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
      {#if effectiveShell}
        <span
          class="shell-badge"
          class:pwsh={effectiveShell.is_pwsh}
          class:zsh={effectiveShell.shell_name === 'zsh'}
          class:bash={effectiveShell.shell_name === 'bash'}
          class:fish={effectiveShell.shell_name === 'fish'}
          title={`Executable: ${effectiveShell.executable_path} (Version: ${effectiveShell.version})`}
        >
          {effectiveShell.shell_name}
        </span>
      {/if}
      {#if isConnecting}
        <span class="connecting-label">Connecting PTY...</span>
      {/if}
    </div>

    <div class="header-actions">
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

  .shell-badge.zsh,
  .shell-badge.bash,
  .shell-badge.fish {
    background-color: color-mix(in srgb, var(--color-secondary) 15%, var(--color-surface));
    color: var(--color-secondary);
    border-color: color-mix(in srgb, var(--color-secondary) 35%, transparent);
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
