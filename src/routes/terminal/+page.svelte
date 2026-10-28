<script lang="ts">
  import EndpointModuleShell from "../endpoint/endpoint-module-shell.svelte";
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import TerminalWindow from "../endpoint/terminal-window.svelte";

  const store = endpointStore;
</script>

<EndpointModuleShell title="Terminal">
  <div class="module-fill">
    <TerminalWindow shell={store.shell} />
  </div>
</EndpointModuleShell>

<style>
  .module-fill {
    flex: 1;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
</style>
