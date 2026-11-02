<script lang="ts">
  import EndpointModuleShell from "../endpoint/endpoint-module-shell.svelte";
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import UpdatesWindow from "../endpoint/updates-window.svelte";

  const store = endpointStore;
</script>

<EndpointModuleShell title="Updates">
  <UpdatesWindow {store} />
</EndpointModuleShell>
