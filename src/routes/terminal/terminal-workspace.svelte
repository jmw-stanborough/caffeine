<script lang="ts">
  import { endpointStore } from "../endpoint/endpoint.svelte";
  import TerminalWindow from "../endpoint/terminal-window.svelte";

  let { active = false }: { active?: boolean } = $props();
</script>

<div class="terminal-page">
  <TerminalWindow shell={endpointStore.shell} {active} />
</div>

<style>
  .terminal-page {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: var(--page-padding);
    box-sizing: border-box;
    overflow: hidden;
    background-color: var(--color-base);
  }
</style>
