<script lang="ts">
  import { Printer } from "$lib/components/icons";
  import StatusSummaryCard, {
    type AttentionItem,
  } from "$lib/components/status-summary-card.svelte";
  import { printerStore } from "../../routes/printers/printers.svelte";
  import { getEffectivePrinterStatus, getPrinterIssues } from "../../routes/printers/status";

  let attention = $derived<AttentionItem[]>(
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
</script>

<StatusSummaryCard
  title="Printers"
  href="/printers"
  noun="printer"
  icon={Printer}
  loading={printerStore.loading}
  errorMessage={printerStore.errorMessage}
  total={printerStore.items.length}
  counts={printerStore.statusCounts}
  {attention}
/>
