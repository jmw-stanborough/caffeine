<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
</script>

<div class="admin-warning-banner" role="status">
  <span class="admin-warning-icon">
    <AlertTriangle size={16} />
  </span>
  <span>
    Running without administrator privileges. Elevated permissions may be required to download or install updates.
  </span>
</div>

<style>
  .admin-warning-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: color-mix(in srgb, var(--color-warning) 12%, var(--color-surface));
    border: solid var(--width-border) color-mix(in srgb, var(--color-warning) 30%, transparent);
    border-radius: 0.375rem;
    margin-bottom: 0.75rem;
    color: var(--color-text);
    font-size: 0.78rem;
    flex-shrink: 0;
  }

  .admin-warning-icon {
    color: var(--color-warning);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }
</style>
