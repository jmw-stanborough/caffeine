<script lang="ts">
  interface Agent {
    port: number;
    api_key: string;
  }

  interface Server {
    host: string;
    label: string;
    description?: string;
    agent?: Agent;
  }

  let { 
    server, 
    onclick 
  }: { 
    server: Server; 
    onclick: () => void; 
  } = $props();
</script>

<article 
  class="server-card" 
  {onclick}
  role="button"
  tabindex="0"
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onclick(); }}
>
  <div class="card-main">
    <div class="card-title-row">
      <span
        class="status-indicator"
        class:has-agent={Boolean(server.agent)}
        title={server.agent ? "Agent active" : "Unmonitored"}
      ></span>
      <h2 class="server-label">{server.label}</h2>
      {#if server.agent}
        <span class="badge-agent">Agent :{server.agent.port}</span>
      {/if}
    </div>

    <p class="server-description">{server.description ?? "No description"}</p>
  </div>

  <footer class="card-footer">
    <code class="server-host">{server.host}</code>
  </footer>
</article>

<style>
  .server-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 8.5rem;
    padding: 1rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    background-color: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .server-card:hover {
    border-color: var(--color-primary);
  }

  .server-card:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .card-main {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-indicator {
    width: 0.625rem;
    height: 0.625rem;
    border-radius: 50%;
    background-color: var(--color-text-muted);
  }

  .status-indicator.has-agent {
    background-color: var(--color-primary);
  }

  .server-label {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .badge-agent {
    margin-left: auto;
    padding: 0.15rem 0.4rem;
    font-size: 0.75rem;
    font-weight: 600;
    border-radius: 0.25rem;
    background-color: color-mix(in srgb, var(--color-secondary) 15%, transparent);
    color: var(--color-secondary);
    border: solid var(--width-border) color-mix(in srgb, var(--color-secondary) 40%, transparent);
  }

  .server-description {
    margin: 0;
    font-size: 0.95rem;
    color: var(--color-text);
    line-height: 1.4;
  }

  .card-footer {
    margin-top: 1rem;
    padding-top: 0.75rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .server-host {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }
</style>
