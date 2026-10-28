<script lang="ts">
  import EndpointModuleShell from "../endpoint/endpoint-module-shell.svelte";
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import UpdatesWindow from "../endpoint/updates-window.svelte";

  const store = endpointStore;
</script>

<EndpointModuleShell title="Updates">
  <div class="module-fill">
    <UpdatesWindow
      updates={store.updates}
      isChecking={store.updatesChecking}
      downloadingId={store.downloadingId}
      installingId={store.installingId}
      operationStatus={store.operationStatus}
      errorMessage={store.updatesError}
      isAdmin={store.isAdmin}
      onCheck={() => store.checkForUpdates()}
      onDownload={(id) => store.downloadUpdates(id)}
      onInstall={(id) => store.installUpdates(id)}
    />
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
