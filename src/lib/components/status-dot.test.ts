import { describe, expect, it } from "vitest";
import { render } from "svelte/server";
import StatusDot, {
  resolveStatusDotVariant,
  type StatusDotStatus,
} from "./status-dot.svelte";

describe("StatusDot component", () => {
  it("resolves semantic and monitor status aliases to canonical variants", () => {
    const cases: [StatusDotStatus, string][] = [
      ["success", "success"],
      ["online", "success"],
      ["active", "success"],
      ["warning", "warning"],
      ["degraded", "warning"],
      ["error", "error"],
      ["offline", "error"],
      ["neutral", "neutral"],
      ["checking", "neutral"],
      ["inactive", "neutral"],
    ];

    for (const [status, expected] of cases) {
      expect(resolveStatusDotVariant(status)).toBe(expected);
    }
  });

  it("renders success state with status-dot and success classes", () => {
    const { html } = render(StatusDot, { props: { status: "success" } });
    expect(html).toContain('class="status-dot success"');
    expect(html).toContain('width="8"');
    expect(html).toContain('height="8"');
  });

  it("renders warning state with status-dot and warning classes", () => {
    const { html } = render(StatusDot, { props: { status: "warning" } });
    expect(html).toContain('class="status-dot warning"');
  });

  it("renders error state with status-dot and error classes", () => {
    const { html } = render(StatusDot, { props: { status: "error" } });
    expect(html).toContain('class="status-dot error"');
  });

  it("preserves alias class alongside canonical variant class", () => {
    const { html } = render(StatusDot, { props: { status: "checking", size: 10 } });
    expect(html).toContain('class="status-dot neutral checking"');
    expect(html).toContain('width="10"');
    expect(html).toContain('height="10"');
  });

  it("forwards title, aria-label, and custom class", () => {
    const { html } = render(StatusDot, {
      props: {
        status: "online",
        title: "Active",
        class: "custom-dot",
        "aria-label": "Server online",
      } as Record<string, unknown>,
    });
    expect(html).toContain("<title>Active</title>");
    expect(html).toContain('class="status-dot success online custom-dot"');
    expect(html).toContain('aria-label="Server online"');
    expect(html).not.toContain('aria-hidden="true"');
  });
});
