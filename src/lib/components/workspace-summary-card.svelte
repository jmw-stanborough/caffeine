<script lang="ts">
  import { AlertTriangle, Briefcase } from "$lib/components/icons";
  import StatusDot from "$lib/components/status-dot.svelte";
  import { notesStore } from "../../routes/notes/notes.svelte";
  import { scriptsStore } from "../../routes/scripts/scripts.svelte";
  import { todoStore } from "../../routes/todo/todo.svelte";

  let overdueCount = $derived(todoStore.statusCounts.offline);
  let warningCount = $derived(todoStore.statusCounts.degraded);
  let activeCount = $derived(todoStore.statusCounts.online);
  let totalActive = $derived(overdueCount + warningCount + activeCount);
  let hasOverdue = $derived(overdueCount > 0);
  let hasWarning = $derived(!hasOverdue && warningCount > 0);
  let isAllOnTrack = $derived(totalActive > 0 && !hasOverdue && !hasWarning);

  let errorMessage = $derived(
    todoStore.errorMessage ?? notesStore.errorMessage ?? scriptsStore.errorMessage,
  );
  let isInitialLoading = $derived(
    todoStore.loading &&
      todoStore.items.length === 0 &&
      notesStore.loading &&
      scriptsStore.loading,
  );
</script>

<section class="card" aria-label="Workspace summary">
  <a href="/todo" class="card-header-link">
    <h2 class="card-header">
      <Briefcase size={20} />
      <span>Workspace</span>
    </h2>
  </a>

  {#if errorMessage}
    <div class="error-banner">{errorMessage}</div>
  {:else if isInitialLoading}
    <p class="status-message">Loading…</p>
  {:else}
    <a href="/todo" class="headline-link">
      <p
        class="headline"
        class:ok={isAllOnTrack}
        class:warn={hasWarning}
        class:crit={hasOverdue}
      >
        {#if hasOverdue || hasWarning}
          <AlertTriangle size={16} />
        {/if}
        <span>
          {#if hasOverdue}
            {overdueCount} overdue {overdueCount === 1 ? "task" : "tasks"}{warningCount > 0
              ? `, ${warningCount} approaching due`
              : ""}
          {:else if hasWarning}
            {warningCount} {warningCount === 1 ? "task" : "tasks"} approaching due
          {:else if totalActive > 0}
            All {totalActive} active {totalActive === 1 ? "task" : "tasks"} on track
          {:else}
            All active tasks on track
          {/if}
        </span>
      </p>
    </a>

    <dl class="stats">
      <div class="stat" class:dim={activeCount === 0}>
        <dt class="stat-label"><StatusDot status="online" size={8} />Active Tasks</dt>
        <dd class="stat-value">
          <a href="/todo" class="stat-link">{activeCount}</a>
        </dd>
      </div>

      <div class="stat" class:dim={warningCount === 0}>
        <dt class="stat-label"><StatusDot status="degraded" size={8} />Approaching Due</dt>
        <dd class="stat-value">
          <a href="/todo" class="stat-link">{warningCount}</a>
        </dd>
      </div>

      <div class="stat" class:dim={overdueCount === 0}>
        <dt class="stat-label"><StatusDot status="offline" size={8} />Overdue</dt>
        <dd class="stat-value">
          <a href="/todo" class="stat-link">{overdueCount}</a>
        </dd>
      </div>

      <div class="stat" class:dim={todoStore.completedCount === 0}>
        <dt class="stat-label"><StatusDot status="neutral" size={8} />Completed</dt>
        <dd class="stat-value">
          <a href="/todo" class="stat-link">{todoStore.completedCount}</a>
        </dd>
      </div>

      <div class="stat" class:dim={notesStore.notes.length === 0}>
        <dt class="stat-label">
          <a href="/notes" class="stat-label-link">Saved Notes</a>
        </dt>
        <dd class="stat-value">
          <a href="/notes" class="stat-link">{notesStore.notes.length}</a>
        </dd>
      </div>

      <div class="stat" class:dim={scriptsStore.scripts.length === 0}>
        <dt class="stat-label">
          <a href="/scripts" class="stat-label-link">Saved Scripts</a>
        </dt>
        <dd class="stat-value">
          <a href="/scripts" class="stat-link">{scriptsStore.scripts.length}</a>
        </dd>
      </div>
    </dl>

    {#if todoStore.attentionItems.length > 0}
      <ul class="attention-list">
        {#each todoStore.attentionItems as item (item.id)}
          <li>
            <a href="/todo" class="attention-item">
              <StatusDot status={item.severity} size={8} />
              <span class="attention-label">{item.label}</span>
              <span class="attention-detail">{item.detail}</span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</section>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border: solid var(--width-border) var(--color-border);
    border-radius: 8px;
    background-color: var(--color-surface);
    color: var(--color-text);
    transition: background-color 0.15s ease, border-color 0.15s ease;
  }

  .card:hover {
    border-color: var(--color-primary);
  }

  .card-header-link,
  .headline-link,
  .stat-link,
  .stat-label-link {
    color: inherit;
    text-decoration: none;
  }

  .card-header-link:focus-visible,
  .headline-link:focus-visible,
  .stat-link:focus-visible,
  .stat-label-link:focus-visible,
  .attention-item:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
    border-radius: 4px;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 1.1rem;
    color: var(--color-primary);
  }

  .headline {
    margin: 0;
    font-weight: 600;
    color: var(--color-text);
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .headline.ok {
    color: var(--color-success);
  }

  .headline.warn {
    color: var(--color-warning);
  }

  .headline.crit {
    color: var(--color-failure);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(105px, 1fr));
    gap: 8px;
    margin: 0;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat.dim {
    opacity: 0.5;
  }

  .stat-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .stat-label-link:hover,
  .stat-link:hover {
    color: var(--color-primary);
  }

  .stat-value {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .attention-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 12px 0 0;
    list-style: none;
    border-top: solid var(--width-border) var(--color-border);
  }

  .attention-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 0.9rem;
    color: inherit;
    text-decoration: none;
  }

  .attention-item:hover .attention-label {
    color: var(--color-primary);
  }

  .attention-label {
    font-weight: 600;
  }

  .attention-detail {
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
