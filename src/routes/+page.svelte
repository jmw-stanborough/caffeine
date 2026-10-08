<script lang="ts">
  // import { invoke } from "@tauri-apps/api/core";
</script>

<section class="cards">
  <div class="card">
    <h1 class="card-header">Welcome to Caffeine!</h1>
    <p class="card-body">This is the dashboard. Please use the navigation bar to access your desired function.</p>
    <p class="card-footer">More to come soon!</p>
  </div>
</section>

<style>
  .cards {
    margin: 16px auto;
    padding-inline: 16px;
    max-width: var(--width-content);
  }

  .card {
    padding: 16px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;

    background-color: var(--color-surface);
  }

  .card-header {
    margin: 0;

    color: var(--color-primary);
  }

  .card-body {
    color: var(--color-text);
  }

  .card-footer {
    color: var(--color-text-muted);
  }
</style>
