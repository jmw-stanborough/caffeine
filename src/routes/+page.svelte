<script lang="ts">
</script>

<div class="dashboard-container">
  <header class="dashboard-header">
    <h1 class="page-title">Dashboard</h1>
  </header>

  <section class="cards-grid">
    <div class="card">
      <h2 class="card-header">Welcome to Caffeine</h2>
      <p class="card-body">Select a module from the sidebar to begin.</p>
      <span class="card-footer">More features to come...</span>
    </div>
  </section>
</div>

<style>
  .dashboard-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
    gap: 16px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;
    background-color: var(--color-surface);
  }

  .card-header {
    margin: 0;
    font-size: 1.1rem;
    color: var(--color-primary);
  }

  .card-body {
    margin: 0;
    color: var(--color-text);
    line-height: 1.4;
  }

  .card-footer {
    margin-top: auto;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
</style>
