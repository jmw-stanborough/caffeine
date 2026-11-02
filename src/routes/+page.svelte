<script lang="ts">
  import { getDashboardWidgets, type DashboardWidget } from "$lib/features";

  let widgets = $derived<DashboardWidget[]>(getDashboardWidgets());
</script>

<div class="dashboard-container">
  <header class="dashboard-header">
    <h1 class="page-title">Dashboard</h1>
  </header>

  <section class="cards-grid">
    {#each widgets as { id, component: Widget } (id)}
      <Widget />
    {/each}
  </section>
</div>

<style>
  .dashboard-container {
    display: flex;
    flex-direction: column;
    gap: var(--page-gap);
    padding: var(--page-padding);
  }

  .dashboard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 380px), 1fr));
    gap: 16px;
  }
</style>
