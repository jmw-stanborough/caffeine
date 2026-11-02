<script lang="ts">
  import type { Snippet } from "svelte";
  import type { SVGAttributes } from "svelte/elements";

  interface Props extends SVGAttributes<SVGSVGElement> {
    size?: number | string;
    filled?: boolean;
    fill?: string;
    stroke?: string;
    strokeWidth?: number | string;
    r?: number;
    text?: string | number;
    textColor?: string;
    children?: Snippet;
  }

  let {
    size = 16,
    filled = true,
    fill = undefined,
    stroke = undefined,
    strokeWidth = 2,
    r = undefined,
    text = undefined,
    textColor = "var(--color-base)",
    children = undefined,
    class: className = undefined,
    ...restProps
  }: Props = $props();

  let effectiveFill = $derived(
    fill ?? (filled ? "currentColor" : "none"),
  );
  let effectiveStroke = $derived(
    stroke ?? (filled ? "none" : "currentColor"),
  );
  let effectiveR = $derived(
    r ?? (effectiveStroke !== "none" ? 10 : 11),
  );

  let textStr = $derived(text !== undefined ? String(text) : "");
  let fontSize = $derived(
    textStr.length > 2 ? 10 : textStr.length > 1 ? 12 : 14,
  );
  let isAriaHidden = $derived(
    restProps["aria-label"] ? undefined : (restProps["aria-hidden"] ?? "true"),
  );
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={effectiveFill}
  stroke={effectiveStroke}
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden={isAriaHidden}
  {...restProps}
>
  <circle cx="12" cy="12" r={effectiveR} />
  {#if textStr}
    <text
      x="12"
      y="12"
      text-anchor="middle"
      dominant-baseline="central"
      fill={textColor}
      stroke="none"
      font-size={fontSize}
      font-weight="700"
      font-family="inherit"
    >{textStr}</text>
  {/if}
  {#if children}
    {@render children()}
  {/if}
</svg>
