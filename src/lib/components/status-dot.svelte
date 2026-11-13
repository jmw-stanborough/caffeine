<script lang="ts" module>
  export type StatusDotVariant = "success" | "warning" | "error" | "neutral";

  export type StatusDotStatus =
    | StatusDotVariant
    | "online"
    | "degraded"
    | "offline"
    | "checking"
    | "active"
    | "inactive";

  const STATUS_VARIANT_MAP: Record<StatusDotStatus, StatusDotVariant> = {
    success: "success",
    online: "success",
    active: "success",
    warning: "warning",
    degraded: "warning",
    error: "error",
    offline: "error",
    neutral: "neutral",
    checking: "neutral",
    inactive: "neutral",
  };

  export function resolveStatusDotVariant(status: StatusDotStatus): StatusDotVariant {
    return STATUS_VARIANT_MAP[status];
  }
</script>

<script lang="ts">
  import { Circle } from "$lib/components/icons";

  interface Props {
    status?: StatusDotStatus;
    size?: number | string;
    title?: string;
    class?: string;
    "aria-label"?: string;
    "aria-hidden"?: boolean | "true" | "false";
  }

  let {
    status = "neutral",
    size = 8,
    title = undefined,
    class: className = undefined,
    "aria-label": ariaLabel = undefined,
    "aria-hidden": ariaHidden = undefined,
  }: Props = $props();

  let variant = $derived(resolveStatusDotVariant(status));
  let combinedClass = $derived(
    ["status-dot", variant, status !== variant ? status : undefined, className]
      .filter(Boolean)
      .join(" "),
  );
</script>

<Circle
  {size}
  {title}
  class={combinedClass}
  aria-label={ariaLabel}
  aria-hidden={ariaHidden}
/>

<style>
  :global(.status-dot) {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  :global(.status-dot.success) {
    color: var(--color-success);
  }

  :global(.status-dot.warning) {
    color: var(--color-warning);
    animation: status-dot-pulse 1.8s infinite ease-in-out;
  }

  :global(.status-dot.error) {
    color: var(--color-failure);
    animation: status-dot-pulse 0.9s infinite ease-in-out;
  }

  @keyframes status-dot-pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.4;
      transform: scale(0.8);
    }
  }
</style>
