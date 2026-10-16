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
      description: "Primary DHCP/DNS server",
    },
    {
      host: "VSVR-005.EDU.LAN",
      label: "VSVR-005",
      description: "Shared data",
    },
    {
      host: "VSVR-006.EDU.LAN",
      label: "VSVR-006",
      description: "User data",
    },
    {
      host: "VSVR-008.EDU.LAN",
      label: "VSVR-008",
      description: "Application server",
    }
  ]
</script>

<h1 class="page-header text-center">Servers</h1>

<section class="server-cards mx-center">
  {#each servers as server}
    <div class="server-card">
      <h2 class="server-card-header text-center">{server.label}</h2>
      <p class="server-card-description text-center">{server.description}</p>
      <footer class="text-center server-card-footer">
        <small class="text-muted">
          {server.host}
        </small>
      </footer>
    </div>
  {/each}
</section>

<style>
  .server-cards {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;

    max-width: var(--width-content);

    margin-inline: auto;
    padding: 4px;
  }

  .server-card {
    display: flex;
    flex-direction: column;

    min-height: 8rem;

    padding: 4px;

    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;
  }

  .server-card-header {
    margin: 0;
  }

  .server-card-description {
    margin-top: 8px;
  }

  .server-card-footer {
    margin-top: auto;
  }
</style>
