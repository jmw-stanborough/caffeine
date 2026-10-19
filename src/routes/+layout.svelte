<script>
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Sidebar from "$lib/sidebar.svelte";

  let { children } = $props();
</script>

<div class="app-shell">
  <Sidebar />
  <main class="content-area">
    {@render children()}
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .content-area {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    background-color: var(--color-base);
  }
</style>
