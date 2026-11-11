import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import Circle from "./Circle.svelte";
import { Circle as ExportedCircle } from "./index";

describe("Circle icon component", () => {
  it("exports correctly from index file", () => {
    expect(ExportedCircle).toBe(Circle);
  });

  it("renders a filled circle by default", () => {
    const { html } = render(Circle, { props: { size: 10 } });
    expect(html).toContain('<circle cx="12" cy="12" r="11"');
    expect(html).toContain('fill="currentColor"');
    expect(html).toContain('stroke="none"');
    expect(html).toContain('width="10"');
    expect(html).toContain('height="10"');
  });

  it("renders an outlined circle when filled is false", () => {
    const { html } = render(Circle, { props: { size: 16, filled: false } });
    expect(html).toContain('<circle cx="12" cy="12" r="10"');
    expect(html).toContain('fill="none"');
    expect(html).toContain('stroke="currentColor"');
    expect(html).toContain('stroke-width="2"');
  });

  it("renders text centered inside circle when text prop is provided", () => {
    const { html } = render(Circle, { props: { size: 15, text: 3 } });
    expect(html).toContain("<text");
    expect(html).toContain('text-anchor="middle"');
    expect(html).toContain('dominant-baseline="central"');
    expect(html).toContain(">3</text>");
  });

  it("applies aria-hidden by default, but respects aria-label when supplied", () => {
    const defaultOutput = render(Circle, { props: {} });
    expect(defaultOutput.html).toContain('aria-hidden="true"');

    const labeledOutput = render(Circle, {
      props: { text: 1, "aria-label": "1 alert" } as Record<string, unknown>,
    });
    expect(labeledOutput.html).toContain('aria-label="1 alert"');
    expect(labeledOutput.html).not.toContain('aria-hidden="true"');
  });

  it("renders SVG title element when title prop is supplied", () => {
    const { html } = render(Circle, { props: { title: "Active" } });
    expect(html).toContain("<title>Active</title>");
  });
});
