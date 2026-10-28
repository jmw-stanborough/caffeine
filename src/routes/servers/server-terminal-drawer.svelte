<script lang="ts">
  import TerminalView from "$lib/terminal/terminal-view.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import type { Server } from "./types";

  interface Props {
    server: Server;
    onclose: () => void;
  }

  let { server, onclose }: Props = $props();

  let terminalView = $state<ReturnType<typeof TerminalView> | null>(null);
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let isConnecting = $state(true);
  let connectionError = $state<string | null>(null);

  const effectivePort = $derived(server.ssh_port ?? 22);
  const effectiveUser = $derived(server.remote_username || null);

  async function startSession() {
    if (!terminalView) return;
    connectionError = null;
    isConnecting = true;

    const dims = terminalView.getDimensions();

    try {
      await invoke("start_server_ssh_terminal", {
        serverId: server.id,
        host: server.host,
        port: effectivePort,
        username: effectiveUser,
        cols: dims.cols,
        rows: dims.rows,
      });
      isConnecting = false;
    } catch (err: unknown) {
      connectionError = String(err);
      terminalView.writeln(`\r\n\x1b[31mFailed to launch SSH session: ${err}\x1b[0m\r\n`);
      isConnecting = false;
      return;
    }

    const dataEvent = `server-terminal://${server.id}/data`;
    const exitEvent = `server-terminal://${server.id}/exit`;

    unlistenData = await listen<string>(dataEvent, (event) => {
      terminalView?.write(event.payload);
    });

    unlistenExit = await listen(exitEvent, () => {
      terminalView?.writeln("\r\n\x1b[33m[SSH session closed]\x1b[0m\r\n");
    });
  }

  function handleData(data: string) {
    invoke("write_server_ssh_terminal", {
      serverId: server.id,
      data,
    }).catch((err) => {
      console.error("Failed to send terminal data:", err);
    });
  }

  function handleResize(cols: number, rows: number) {
    invoke("resize_server_ssh_terminal", {
      serverId: server.id,
      cols,
      rows,
    }).catch((err) => {
      console.error("Failed to resize terminal session:", err);
    });
  }

  function handleInterrupt() {
    handleData("\x03");
  }

  function handleClear() {
    terminalView?.clear();
  }

  async function handleRestart() {
    cleanupSession();
    terminalView?.clear();
    await startSession();
  }

  function cleanupSession() {
    if (unlistenData) {
      unlistenData();
      unlistenData = null;
    }
    if (unlistenExit) {
      unlistenExit();
      unlistenExit = null;
    }
    invoke("close_server_ssh_terminal", { serverId: server.id }).catch((err) => {
      console.error("Failed to close SSH session:", err);
    });
  }

  onMount(() => {
    startSession();
  });

  onDestroy(() => {
    cleanupSession();
  });
</script>

<aside class="terminal-drawer" aria-label="Server SSH Terminal">
  <div class="drawer-header">
    <div class="header-info">
      <div class="terminal-icon" aria-hidden="true">
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
      <div class="title-group">
        <h2 class="drawer-title">{server.label}</h2>
        <span class="host-badge">{server.host}:{effectivePort}</span>
        {#if effectiveUser}
          <span class="user-badge">{effectiveUser}</span>
        {/if}
      </div>
      {#if isConnecting}
        <span class="status-indicator connecting">Connecting...</span>
      {/if}
    </div>

    <div class="header-actions">
      <button
        type="button"
        class="btn-action interrupt"
        onclick={handleInterrupt}
        title="Send Ctrl+C"
      >
        Ctrl+C
      </button>

      <button
        type="button"
        class="btn-action"
        onclick={handleClear}
        title="Clear screen"
      >
        Clear
      </button>

      <button
        type="button"
        class="btn-action"
        onclick={handleRestart}
        title="Restart SSH session"
      >
        Restart
      </button>

      <div class="divider" aria-hidden="true"></div>

      <button
        type="button"
        class="btn-close"
        onclick={onclose}
        aria-label="Close terminal"
      >
        ✕
      </button>
    </div>
  </div>

  <div class="drawer-body">
    <TerminalView
      bind:this={terminalView}
      onData={handleData}
      onResize={handleResize}
    />
  </div>
</aside>

<style>
  .terminal-drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(48rem, 90vw);
    background-color: var(--color-base);
    border-left: solid var(--width-border) var(--color-border);
    box-shadow: -8px 0 24px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
    z-index: 900;
  }

  .drawer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.65rem 1rem;
    background-color: var(--color-surface);
    border-bottom: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    min-width: 0;
  }

  .terminal-icon {
    color: var(--color-primary);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .drawer-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .host-badge,
  .user-badge {
    font-size: 0.72rem;
    font-family: monospace;
    font-weight: 700;
    padding: 0.12rem 0.4rem;
    border-radius: 0.25rem;
    background-color: var(--color-surface-hover);
    color: var(--color-text-muted);
    border: solid 1px var(--color-border);
    white-space: nowrap;
  }

  .status-indicator.connecting {
    font-size: 0.75rem;
    color: var(--color-secondary);
    font-style: italic;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .btn-action {
    background-color: transparent;
    border: solid 1px var(--color-border);
    color: var(--color-text-muted);
    padding: 0.2rem 0.55rem;
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

  .btn-action.interrupt {
    color: var(--color-failure);
    border-color: color-mix(in srgb, var(--color-failure) 40%, transparent);
  }

  .btn-action.interrupt:hover {
    background-color: var(--color-failure);
    color: var(--color-base);
  }

  .divider {
    width: 1px;
    height: 16px;
    background-color: var(--color-border);
    margin: 0 0.2rem;
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
    padding: 0.2rem 0.4rem;
    border-radius: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.12s ease, color 0.12s ease;
  }

  .btn-close:hover {
    background-color: var(--color-surface-hover);
    color: var(--color-text);
  }

  .drawer-body {
    flex: 1;
    min-height: 0;
    padding: 0.5rem;
    background-color: var(--color-base);
    overflow: hidden;
  }
</style>
