<script>
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Navbar from "$lib/navbar.svelte";

  let { children } = $props();
</script>

<header class="navigation-bar">
  <Navbar />
</header>

<main class="main-content">
  {@render children()}
</main>

<style>
  .navigation-bar {
    position: sticky;
    top: 0;
  }

  .main-content {
    margin-inline: 2vw;
  }
</style>
