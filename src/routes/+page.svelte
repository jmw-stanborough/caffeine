<script lang="ts">
  import StatusSummaryCard, {
    type AttentionItem,
  } from "$lib/components/status-summary-card.svelte";
  import { printerStore } from "./printers/printers.svelte";
  import { getEffectivePrinterStatus, getPrinterIssues } from "./printers/status";
  import { serverStore } from "./servers/servers.svelte";

  function averageLatency(
    states: { status: string; latencyMs?: number }[],
  ): string {
    const samples = states
      .filter((s) => s.status !== "offline" && s.latencyMs !== undefined)
      .map((s) => s.latencyMs as number);
    if (samples.length === 0) return "—";
    const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
    return `${Math.round(mean)} ms`;
  }

  let serverAttention = $derived<AttentionItem[]>(
    serverStore.items.flatMap((server) => {
      const status = serverStore.statusMap[server.id]?.status;
      if (status !== "offline" && status !== "degraded") return [];
      return [
        {
          id: server.id,
          label: server.label || server.host,
          detail: status === "offline" ? "Offline" : "Degraded",
          severity: status,
        },
      ];
    }),
  );

  let printerAttention = $derived<AttentionItem[]>(
    printerStore.items.flatMap((printer) => {
      const state = printerStore.statusMap[printer.id];
      const severity = getEffectivePrinterStatus(state);
      if (severity !== "offline" && severity !== "degraded") return [];
      return [
        {
          id: printer.id,
          label: printer.label || printer.host,
          detail: getPrinterIssues(state).join(", "),
          severity,
        },
      ];
    }),
  );

  let serverLatency = $derived(
    averageLatency(Object.values(serverStore.statusMap)),
  );
</script>

<div class="dashboard-container">
  <header class="dashboard-header">
    <h1 class="page-title">Dashboard</h1>
  </header>

  <section class="cards-grid">
    <StatusSummaryCard
      title="Servers"
      href="/servers"
      noun="server"
      loading={serverStore.loading}
      errorMessage={serverStore.errorMessage}
      total={serverStore.items.length}
      counts={serverStore.statusCounts}
      attention={serverAttention}
      extraStat={{ label: "Avg latency", value: serverLatency }}
    />
    <StatusSummaryCard
      title="Printers"
      href="/printers"
      noun="printer"
      loading={printerStore.loading}
      errorMessage={printerStore.errorMessage}
      total={printerStore.items.length}
      counts={printerStore.statusCounts}
      attention={printerAttention}
    />
  </section>
</div>

<style>
  .dashboard-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 380px), 1fr));
    gap: 16px;
  }
</style>
