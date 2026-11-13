import { describe, expect, it } from "vitest";
import { renderMarkdownToHtml } from "./markdown";

describe("renderMarkdownToHtml", () => {
  it("renders headings from h1 to h6", () => {
    const md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    const html = renderMarkdownToHtml(md);
    expect(html).toContain("<h1>H1</h1>");
    expect(html).toContain("<h2>H2</h2>");
    expect(html).toContain("<h3>H3</h3>");
    expect(html).toContain("<h4>H4</h4>");
    expect(html).toContain("<h5>H5</h5>");
    expect(html).toContain("<h6>H6</h6>");
  });

  it("renders bold, italic, strikethrough, inline code, and paragraphs", () => {
    const html = renderMarkdownToHtml(
      "Hello **bold** and *italic* and ~~gone~~ with `const x = 1 < 2`.",
    );
    expect(html).toBe(
      "<p>Hello <strong>bold</strong> and <em>italic</em> and <del>gone</del> with <code>const x = 1 &lt; 2</code>.</p>",
    );
  });

  it("renders GFM tables with column alignments", () => {
    const md = [
      "| Name | Count | Status |",
      "| :--- | :---: | ---: |",
      "| Alpha | 10 | OK |",
    ].join("\n");

    const html = renderMarkdownToHtml(md);
    expect(html).toContain("<table><thead><tr>");
    expect(html).toContain('<th style="text-align:left">Name</th>');
    expect(html).toContain('<th style="text-align:center">Count</th>');
    expect(html).toContain('<th style="text-align:right">Status</th>');
    expect(html).toContain('<td style="text-align:center">10</td>');
  });

  it("renders fenced code blocks with syntax highlighting", () => {
    const md = ["```ps1", "if ($ok) {", '  $msg = "hi"', "}", "```"].join("\n");
    const html = renderMarkdownToHtml(md);
    expect(html).toContain('<pre class="md-code-block"><code class="language-powershell">');
    expect(html).toContain('<span class="hl-keyword">if</span>');
    expect(html).toContain('<span class="hl-variable">$ok</span>');
    expect(html).toContain('<span class="hl-string">&quot;hi&quot;</span>');
  });

  it("renders unordered, ordered, nested, and task lists", () => {
    const md = [
      "- [ ] Pending task",
      "- [x] Completed task",
      "  - Nested item",
      "",
      "1. First",
      "2. Second",
    ].join("\n");

    const html = renderMarkdownToHtml(md);
    expect(html).toContain('<ul class="task-list">');
    expect(html).toContain('<input type="checkbox" disabled /> <span>Pending task</span>');
    expect(html).toContain('<input type="checkbox" disabled checked /> <span>Completed task</span>');
    expect(html).toContain("<ul><li>Nested item</li></ul>");
    expect(html).toContain("<ol><li>First</li><li>Second</li></ol>");
  });

  it("renders blockquotes and horizontal rules", () => {
    const md = ["> Quote line", "> **Strong** quote", "", "---"].join("\n");
    const html = renderMarkdownToHtml(md);
    expect(html).toContain("<blockquote><p>Quote line <strong>Strong</strong> quote</p></blockquote>");
    expect(html).toContain("<hr />");
  });

  it("sanitizes raw HTML tags, event attributes, and unsafe link schemes", () => {
    const md = [
      "<script>alert(1)</script>",
      '<img src="x" onerror="alert(2)" />',
      "[Click](javascript:alert(3))",
      "[Safe](https://example.com/docs)",
    ].join("\n\n");

    const html = renderMarkdownToHtml(md);
    expect(html).not.toContain("<script>");
    expect(html).not.toContain("<img");
    expect(html).not.toContain("javascript:");
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(html).toContain("onerror=&quot;alert(2)&quot;");
    expect(html).toContain("<p>Click</p>");
    expect(html).toContain(
      '<a href="https://example.com/docs" target="_blank" rel="noopener noreferrer">Safe</a>',
    );
  });
});
