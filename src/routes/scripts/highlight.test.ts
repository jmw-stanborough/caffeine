import { describe, expect, it } from "vitest";
import { escapeHtml, highlightCode, normalizeHighlightLanguage } from "./highlight";

describe("normalizeHighlightLanguage", () => {
  it("normalizes common script language aliases", () => {
    expect(normalizeHighlightLanguage("ps1")).toBe("powershell");
    expect(normalizeHighlightLanguage("pwsh")).toBe("powershell");
    expect(normalizeHighlightLanguage("batch")).toBe("bat");
    expect(normalizeHighlightLanguage("py")).toBe("python");
    expect(normalizeHighlightLanguage("shell")).toBe("sh");
  });
});

describe("highlightCode", () => {
  it("escapes raw HTML entities across highlighted and plain code", () => {
    const html = highlightCode('echo "<script>alert(1)</script>"', "sh");
    expect(html).not.toContain("<script>");
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(escapeHtml(`a&b<c>d"e'f`)).toBe("a&amp;b&lt;c&gt;d&quot;e&#39;f");
  });

  it("highlights PowerShell keywords, variables, strings, numbers, operators, and comments", () => {
    const code = [
      "# Check service status",
      "if ($count -eq 42) {",
      '  $msg = "ok"',
      "}",
    ].join("\n");

    const html = highlightCode(code, "powershell");
    expect(html).toContain('<span class="hl-comment"># Check service status</span>');
    expect(html).toContain('<span class="hl-keyword">if</span>');
    expect(html).toContain('<span class="hl-variable">$count</span>');
    expect(html).toContain('<span class="hl-operator">-eq</span>');
    expect(html).toContain('<span class="hl-number">42</span>');
    expect(html).toContain('<span class="hl-variable">$msg</span>');
    expect(html).toContain('<span class="hl-string">&quot;ok&quot;</span>');
  });

  it("highlights CMD and Batch scripts", () => {
    const code = [
      "@echo off",
      "REM Print target",
      'set TARGET="C:\\Logs"',
      "if exist %TARGET% echo 1",
    ].join("\n");

    const html = highlightCode(code, "bat");
    expect(html).toContain('<span class="hl-keyword">@echo</span>');
    expect(html).toContain('<span class="hl-keyword">off</span>');
    expect(html).toContain('<span class="hl-comment">REM Print target</span>');
    expect(html).toContain('<span class="hl-keyword">set</span>');
    expect(html).toContain('<span class="hl-string">&quot;C:\\Logs&quot;</span>');
    expect(html).toContain('<span class="hl-variable">%TARGET%</span>');
    expect(html).toContain('<span class="hl-number">1</span>');
  });

  it("highlights Shell (sh, bash, zsh) scripts", () => {
    const code = [
      "#!/usr/bin/env bash",
      "for item in 1 2; do",
      '  echo "$HOME/${item}"',
      "done",
    ].join("\n");

    for (const lang of ["sh", "bash", "zsh"]) {
      const html = highlightCode(code, lang);
      expect(html).toContain('<span class="hl-comment">#!/usr/bin/env bash</span>');
      expect(html).toContain('<span class="hl-keyword">for</span>');
      expect(html).toContain('<span class="hl-keyword">do</span>');
      expect(html).toContain('<span class="hl-keyword">done</span>');
      expect(html).toContain('<span class="hl-number">1</span>');
      expect(html).toContain('<span class="hl-string">&quot;$HOME/${item}&quot;</span>');
    }
  });

  it("highlights Python scripts", () => {
    const code = [
      "# Compute total",
      "def compute(self, count=10):",
      '    return f"items={count}"',
    ].join("\n");

    const html = highlightCode(code, "python");
    expect(html).toContain('<span class="hl-comment"># Compute total</span>');
    expect(html).toContain('<span class="hl-keyword">def</span>');
    expect(html).toContain('<span class="hl-variable">self</span>');
    expect(html).toContain('<span class="hl-number">10</span>');
    expect(html).toContain('<span class="hl-keyword">return</span>');
    expect(html).toContain('<span class="hl-string">f&quot;items={count}&quot;</span>');
  });

  it("falls back safely to escaped HTML for unknown languages", () => {
    expect(highlightCode("<div>raw</div>", "unknown")).toBe("&lt;div&gt;raw&lt;/div&gt;");
    expect(highlightCode("", "powershell")).toBe("");
  });
});
