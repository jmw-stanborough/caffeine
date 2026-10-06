<script lang="ts">
  // import { invoke } from "@tauri-apps/api/core";
</script>

<section class="cards">
  <div class="card">
    <h1 class="card-header">This is the card header</h1>
    <p class="card-body">This is the card body.</p>
    <p class="card-footer">This is the card footer.</p>
  </div>
</section>

<style>
  .cards {
    margin: 16px auto;
    max-width: var(--width-content);
  }

  .card {
    padding: 16px;

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
