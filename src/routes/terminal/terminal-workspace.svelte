<script lang="ts">
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import TerminalWindow from "../endpoint/terminal-window.svelte";

  let { active = false }: { active?: boolean } = $props();
</script>

<div class="endpoint-page-shell">
  <div class="endpoint-workspace">
    <div class="module-fill">
      <TerminalWindow shell={endpointStore.shell} {active} />
    </div>
  </div>
</div>

<style>
  .endpoint-page-shell {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-base);
  }

  .endpoint-workspace {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
    padding: 0.85rem;
    gap: 0.85rem;
    box-sizing: border-box;
    background-color: var(--color-base);
  }

  .module-fill {
    flex: 1;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
</style>
