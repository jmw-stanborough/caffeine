<script lang="ts">
  interface Agent {
    port: number;
    apiKey: string;
  }

  interface Server {
    host: string;
    label: string;
    description?: string;
    agent?: Agent;
  }

  const servers: Server[] = [
    {
      host: "VSVR-001.EDU.LAN",
      label: "VSVR-001",
      description: "Secondary DHCP/DNS server",
    },
    {
      host: "VSVR-002.EDU.LAN",
      label: "VSVR-002",
      description: "Primary DHCP/DNS server"
    },
    {
      host: "VSVR-005.EDU.LAN",
      label: "VSVR-005",
      description: "Shared data file server"
    },
    {
      host: "VSVR-006.EDU.LAN",
      label: "VSVR-006",
      description: "User data file server"
    },
    {
      host: "VSVR-008.EDU.LAN",
      label: "VSVR-008",
      description: "General-purpose application server"
    },
    {
      host: "VSVR-009.EDU.LAN",
      label: "VSVR-009",
      description: "Salamander application server"
    },
    {
      host: "VSVR-011.EDU.LAN",
      label: "VSVR-011",
      description: "MDT & WDS application server"
    },
    {
      host: "VSVR-012.EDU.LAN",
      label: "VSVR-012",
      description: "Live Register application server"
    },
    {
      host: "VSVR-014.EDU.LAN",
      label: "VSVR-014",
      description: "Print server"
    }
  ];
</script>

<div class="page-container">
  <header class="page-header">
    <div class="header-titles">
      <h1 class="page-title">Servers</h1>
      <span class="server-count">{servers.length} configured</span>
    </div>
  </header>

  <section class="server-grid">
    {#each servers as server}
      <article class="server-card">
        <div class="card-main">
          <div class="card-title-row">
            <span class="status-indicator" title="Agent status"></span>
            <h2 class="server-label">{server.label}</h2>
            {#if server.agent}
              <span class="badge">Agent :{server.agent.port}</span>
            {/if}
          </div>

          <p class="server-description">{server.description ?? "No description"}</p>
        </div>

        <footer class="card-footer">
          <code class="server-host">{server.host}</code>
        </footer>
      </article>
    {/each}
  </section>
</div>

<style>
  .page-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-titles {
    display: flex;
    align-items: baseline;
    gap: 1rem;
  }

  .server-count {
    font-size: 1rem;
    color: var(--color-text-muted);
  }

  .server-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(256px, 1fr));
    gap: 1rem;
    width: 100%;
  }

  .server-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 128px;
    padding: 1rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    background-color: var(--color-surface);
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .server-card:hover {
    border-color: var(--color-primary);
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
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background-color: var(--color-text-muted);
  }

  .server-label {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--color-primary);
  }

  .badge {
    margin-left: auto;
    padding: 2px 6px;
    font-size: 0.75rem;
    border-radius: 4px;
    background-color: var(--color-base);
    color: var(--color-text-muted);
  }

  .server-description {
    margin: 0;
    font-size: 1rem;
    color: var(--color-text);
    line-height: 1.4;
  }

  .card-footer {
    margin-top: 16px;
    padding-top: 12px;
    border-top: solid var(--width-border) var(--color-border);
  }

  .server-host {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }
</style>
