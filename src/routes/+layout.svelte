<script>
  import "@fontsource-variable/roboto";
  import "../app.css";
  import Navbar from "$lib/navbar.svelte";

  let { children } = $props();
</script>

<header class="navigation-bar">
  <Navbar />
</header>

<main>
  {@render children()}
</main>

<style>
  .navigation-bar {
    position: sticky;
    top: 0;
  }
</style>
