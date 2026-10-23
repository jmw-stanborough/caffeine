<script lang="ts">
  import { tick } from "svelte";
  import type { ShellDetectionResult, TerminalEntry } from "./types";

  interface Props {
    shell: ShellDetectionResult | null;
    workingDir: string;
    entries: TerminalEntry[];
    isRunning: boolean;
    onExecute: (cmd: string) => void;
    onClear: () => void;
  }

  let {
    shell,
    workingDir,
    entries,
    isRunning,
    onExecute,
    onClear,
  }: Props = $props();

  let inputVal = $state("");
  let inputElement = $state<HTMLInputElement | null>(null);
  let scrollContainer = $state<HTMLDivElement | null>(null);

  let commandHistory = $state<string[]>([]);
  let historyIndex = $state<number>(-1);

  async function scrollToBottom() {
    await tick();
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      const cmd = inputVal.trim();
      if (!cmd || isRunning) return;

      commandHistory.push(cmd);
      historyIndex = -1;
      inputVal = "";

      onExecute(cmd);
      scrollToBottom();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (commandHistory.length === 0) return;

      if (historyIndex === -1) {
        historyIndex = commandHistory.length - 1;
      } else if (historyIndex > 0) {
        historyIndex--;
      }
      inputVal = commandHistory[historyIndex] ?? "";
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIndex === -1) return;

      if (historyIndex < commandHistory.length - 1) {
        historyIndex++;
        inputVal = commandHistory[historyIndex] ?? "";
      } else {
        historyIndex = -1;
        inputVal = "";
      }
    }
  }

  function runQuickCommand(cmd: string) {
    if (isRunning) return;
    commandHistory.push(cmd);
    onExecute(cmd);
    scrollToBottom();
  }

  $effect(() => {
    // Whenever entries change, scroll down
    if (entries.length > 0) {
      scrollToBottom();
    }
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
    </div>

    <div class="header-actions">
      <div class="quick-commands">
        <button
          class="btn-quick"
          onclick={() => runQuickCommand("Get-Process | Select-Object -First 5")}
          disabled={isRunning}
        >
          ps
        </button>
        <button
          class="btn-quick"
          onclick={() => runQuickCommand("ipconfig")}
          disabled={isRunning}
        >
          ipconfig
        </button>
        <button
          class="btn-quick"
          onclick={() => runQuickCommand("Get-Date")}
          disabled={isRunning}
        >
          date
        </button>
      </div>

      <button
        class="btn-clear"
        onclick={onClear}
        title="Clear terminal buffer"
        disabled={entries.length === 0}
      >
        Clear
      </button>
    </div>
  </div>

  <div class="terminal-body" bind:this={scrollContainer}>
    <div class="terminal-welcome">
      <span>Windows PowerShell Terminal Session &bull; Detected shell: <strong>{shell?.shell_name || "PowerShell"}</strong></span>
      <span class="sub-welcome">Type any command or use the quick buttons above. Directory changes (`cd`) persist.</span>
    </div>

    {#each entries as entry (entry.id)}
      <div class="log-entry">
        <div class="prompt-line">
          <span class="prompt-prefix">PS {entry.working_dir}&gt;</span>
          <span class="prompt-cmd">{entry.command}</span>
          <span class="entry-time">{entry.timestamp}</span>
        </div>

        {#if entry.is_running}
          <div class="running-indicator">
            <span class="mini-spin"></span>
            <span>Executing...</span>
          </div>
        {:else if entry.result}
          {#if entry.result.stdout}
            <pre class="stream-stdout">{entry.result.stdout}</pre>
          {/if}
          {#if entry.result.stderr}
            <pre class="stream-stderr">{entry.result.stderr}</pre>
          {/if}
          {#if entry.result.exit_code !== 0}
            <div class="exit-badge">Exit Code: {entry.result.exit_code}</div>
          {/if}
        {/if}
      </div>
    {/each}

    <!-- Interactive Active Prompt Line -->
    <div class="active-prompt-row">
      <span class="prompt-prefix">PS {workingDir}&gt;</span>
      <input
        type="text"
        bind:this={inputElement}
        bind:value={inputVal}
        onkeydown={handleKeydown}
        disabled={isRunning}
        placeholder={isRunning ? "Command is running..." : "Type PowerShell command and press Enter..."}
        class="prompt-input"
        spellcheck="false"
        autocomplete="off"
        autocapitalize="off"
      />
      {#if isRunning}
        <span class="mini-spin prompt-spin"></span>
      {/if}
    </div>
  </div>
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
    padding: 0.65rem 1rem;
    background-color: #282828;
    border-bottom: solid var(--width-border) #3c3836;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.55rem;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
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
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-family: monospace;
    cursor: pointer;
    transition: background-color 0.12s ease;
  }

  .btn-quick:hover:not(:disabled) {
    background-color: #504945;
  }

  .btn-clear {
    background-color: transparent;
    border: solid 1px #504945;
    color: #a89984;
    padding: 0.2rem 0.55rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-clear:hover:not(:disabled) {
    background-color: #3c3836;
    color: #ebdbb2;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .terminal-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.85rem 1rem;
    font-family: "Cascadia Code", "Consolas", monospace;
    font-size: 0.85rem;
    color: #ebdbb2;
    line-height: 1.45;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .terminal-welcome {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding-bottom: 0.65rem;
    border-bottom: solid 1px #3c3836;
    color: #a89984;
    font-size: 0.8rem;
  }

  .sub-welcome {
    font-size: 0.72rem;
    color: #7c6f64;
  }

  .log-entry {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .prompt-line {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    color: #83a598;
  }

  .prompt-prefix {
    color: #fabd2f;
    font-weight: 600;
    flex-shrink: 0;
    user-select: none;
  }

  .prompt-cmd {
    color: #ebdbb2;
    font-weight: 600;
    word-break: break-all;
  }

  .entry-time {
    margin-left: auto;
    font-size: 0.7rem;
    color: #7c6f64;
    flex-shrink: 0;
  }

  .running-indicator {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    color: #d79921;
    font-size: 0.78rem;
  }

  .mini-spin {
    width: 12px;
    height: 12px;
    border: 2px solid #504945;
    border-top-color: #fabd2f;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .stream-stdout {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    color: #ebdbb2;
    font-family: inherit;
    font-size: 0.82rem;
  }

  .stream-stderr {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--color-failure);
    font-family: inherit;
    font-size: 0.82rem;
  }

  .exit-badge {
    font-size: 0.7rem;
    color: var(--color-failure);
    font-weight: 600;
  }

  .active-prompt-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: auto;
    padding-top: 0.5rem;
  }

  .prompt-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #ebdbb2;
    font-family: inherit;
    font-size: 0.85rem;
    padding: 0;
  }

  .prompt-input::placeholder {
    color: #7c6f64;
    font-style: italic;
  }

  .prompt-spin {
    flex-shrink: 0;
  }
</style>
