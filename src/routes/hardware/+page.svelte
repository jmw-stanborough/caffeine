<script lang="ts">
  import EndpointModuleShell from "../endpoint/endpoint-module-shell.svelte";
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import HardwarePanel from "../endpoint/hardware-panel.svelte";

  const store = endpointStore;
</script>

<EndpointModuleShell title="Hardware" enableTelemetry={true}>
  <HardwarePanel
    machine={store.machine}
    isLoading={store.machineLoading}
    errorMessage={store.machineError}
    onRefresh={() => store.fetchMachineInfo(true)}
  />
</EndpointModuleShell>
