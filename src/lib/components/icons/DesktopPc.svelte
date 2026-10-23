<script lang="ts">
  import type { SVGAttributes } from "svelte/elements";

  interface Props extends SVGAttributes<SVGSVGElement> {
    size?: number | string;
  }

  let {
    size = 16,
    class: className = undefined,
    ...restProps
  }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden="true"
  {...restProps}
>
  <rect x="1" y="4" width="13" height="10" rx="1.5" />
  <line x1="7.5" y1="14" x2="7.5" y2="18" />
  <line x1="4.5" y1="18" x2="10.5" y2="18" />
  <rect x="16" y="3" width="7" height="18" rx="1.5" />
  <line x1="19.5" y1="6" x2="19.5" y2="6.01" />
  <line x1="18" y1="9" x2="21" y2="9" />
  <line x1="18" y1="11.5" x2="21" y2="11.5" />
</svg>
