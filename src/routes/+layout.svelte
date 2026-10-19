<script>
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Navbar from "$lib/navbar.svelte";
  import Sidebar from "$lib/sidebar.svelte";

  let { children } = $props();
</script>

<header class="navigation-bar">
  <Navbar />
</header>

<main class="main-content">
  <Sidebar />
  <section class="children-section">
    {@render children()}
  </section>
</main>

<style>
  :root {
    --width-sidebar: 64px;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  .navigation-bar {
    flex-shrink: 0;
    z-index: 20;
  }

  .main-content {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .children-section {
    flex: 1;
    min-width: 0;
    overflow: auto;
  }
</style>
