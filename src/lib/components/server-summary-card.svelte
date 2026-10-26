<script lang="ts">
  import { Server } from "$lib/components/icons";
  import StatusSummaryCard, {
    type AttentionItem,
  } from "$lib/components/status-summary-card.svelte";
  import { serverStore } from "../../routes/servers/servers.svelte";

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

  let attention = $derived<AttentionItem[]>(
    serverStore.items.flatMap((server) => {
      const status = serverStore.statusMap[server.id]?.status;
      if (status !== "offline" && status !== "degraded") return [];
      return [
        {
          id: server.id,
          label: server.label || server.host,
          detail: status === "offline" ? "Error" : "Warning",
          severity: status,
        },
      ];
    }),
  );

  let serverLatency = $derived(
    averageLatency(Object.values(serverStore.statusMap)),
  );
</script>

<StatusSummaryCard
  title="Servers"
  href="/servers"
  noun="server"
  icon={Server}
  loading={serverStore.loading}
  errorMessage={serverStore.errorMessage}
  total={serverStore.items.length}
  counts={serverStore.statusCounts}
  {attention}
  extraStat={{ label: "Avg latency", value: serverLatency }}
/>
