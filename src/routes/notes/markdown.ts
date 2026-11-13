import { escapeHtml, highlightCode, normalizeHighlightLanguage } from "../scripts/highlight";

type TableAlign = "left" | "center" | "right" | null;

const FENCE_RE = /^ {0,3}(`{3,}|~{3,})([^\n]*)$/;
const HEADING_RE = /^ {0,3}(#{1,6})\s+(.+?)\s*#*$/;
const HR_RE = /^ {0,3}(?:(?:-\s*){3,}|(?:\*\s*){3,}|(?:_\s*){3,})$/;
const BLOCKQUOTE_RE = /^ {0,3}>\s?(.*)$/;
const UL_ITEM_RE = /^(\s*)[-*+]\s+(.*)$/;
const OL_ITEM_RE = /^(\s*)\d+\.\s+(.*)$/;
const TASK_RE = /^\[([ xX])\]\s+(.*)$/;
const TABLE_SEP_RE = /^\s*\|?\s*:?-{3,}:?\s*(?:\|\s*:?-{3,}:?\s*)+\|?\s*$|^\s*\|\s*:?-{3,}:?\s*\|\s*$/;

function isSafeUrl(rawUrl: string): boolean {
  const trimmed = rawUrl.trim();
  if (!trimmed) return false;
  if (trimmed.startsWith("#") || trimmed.startsWith("/")) return true;
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed)) return true;
  const lower = trimmed.toLowerCase();
  return lower.startsWith("http://") || lower.startsWith("https://") || lower.startsWith("mailto:");
}

function renderFormattedText(escaped: string): string {
  return escaped
    .replace(/~~([^~\n]+?)~~/g, "<del>$1</del>")
    .replace(/\*\*([^*\n]+?)\*\*/g, "<strong>$1</strong>")
    .replace(/(^|[^A-Za-z0-9_])__([^_\n]+?)__(?=[^A-Za-z0-9_]|$)/g, "$1<strong>$2</strong>")
    .replace(/\*([^*\n]+?)\*/g, "<em>$1</em>")
    .replace(/(^|[^A-Za-z0-9_])_([^_\n]+?)_(?=[^A-Za-z0-9_]|$)/g, "$1<em>$2</em>");
}

function renderInline(raw: string): string {
  const tokens: string[] = [];
  const stash = (html: string): string => {
    const idx = tokens.length;
    tokens.push(html);
    return `\u0000${idx}\u0000`;
  };

  let text = raw.replace(/(`+)([^`]*?)\1/g, (_match, _ticks, code: string) =>
    stash(`<code>${escapeHtml(code)}</code>`),
  );

  text = text.replace(
    /\[([^\]]+)\]\(((?:[^\s()]|\([^\s()]*\))+)(?:\s+"([^"]*)")?\)/g,
    (_match, label: string, url: string, title?: string) => {
      const renderedLabel = renderFormattedText(escapeHtml(label));
      if (!isSafeUrl(url)) {
        return stash(renderedLabel);
      }
      const safeHref = escapeHtml(url.trim());
      const titleAttr = title ? ` title="${escapeHtml(title)}"` : "";
      const isExternal = /^https?:\/\//i.test(url.trim());
      const relAttr = isExternal ? ' target="_blank" rel="noopener noreferrer"' : "";
      return stash(`<a href="${safeHref}"${titleAttr}${relAttr}>${renderedLabel}</a>`);
    },
  );

  const escaped = escapeHtml(text);
  const formatted = renderFormattedText(escaped);
  return formatted.replace(/\u0000(\d+)\u0000/g, (_m, idx) => tokens[Number(idx)] ?? "");
}

function splitTableRow(line: string): string[] {
  const trimmed = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  const cells: string[] = [];
  let current = "";
  for (let i = 0; i < trimmed.length; i++) {
    if (trimmed[i] === "\\" && trimmed[i + 1] === "|") {
      current += "|";
      i++;
    } else if (trimmed[i] === "|") {
      cells.push(current.trim());
      current = "";
    } else {
      current += trimmed[i];
    }
  }
  cells.push(current.trim());
  return cells;
}

function parseTableAlignments(separatorLine: string): TableAlign[] {
  return splitTableRow(separatorLine).map((cell) => {
    const left = cell.startsWith(":");
    const right = cell.endsWith(":");
    if (left && right) return "center";
    if (right) return "right";
    if (left) return "left";
    return null;
  });
}

function alignAttr(align: TableAlign | undefined): string {
  return align ? ` style="text-align:${align}"` : "";
}

function isTableStart(lines: string[], index: number): boolean {
  if (index + 1 >= lines.length) return false;
  return lines[index].includes("|") && TABLE_SEP_RE.test(lines[index + 1]);
}

function isBlockStart(lines: string[], index: number): boolean {
  const line = lines[index];
  if (!line.trim()) return true;
  if (FENCE_RE.test(line)) return true;
  if (HEADING_RE.test(line)) return true;
  if (HR_RE.test(line)) return true;
  if (BLOCKQUOTE_RE.test(line)) return true;
  if (UL_ITEM_RE.test(line) || OL_ITEM_RE.test(line)) return true;
  return isTableStart(lines, index);
}

function parseList(
  lines: string[],
  startIndex: number,
  baseIndent: number,
  ordered: boolean,
): { html: string; nextIndex: number } {
  const items: string[] = [];
  let hasTask = false;
  let i = startIndex;

  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) break;

    const ulMatch = UL_ITEM_RE.exec(line);
    const olMatch = OL_ITEM_RE.exec(line);
    const match = ordered ? olMatch : ulMatch;
    if (!match) break;

    const indent = match[1].length;
    if (indent < baseIndent) break;
    if (indent > baseIndent) {
      if (items.length === 0) break;
      const nested = parseList(lines, i, indent, Boolean(olMatch));
      items[items.length - 1] = items[items.length - 1].replace(/<\/li>$/, `${nested.html}</li>`);
      i = nested.nextIndex;
      continue;
    }

    const content = match[2];
    const taskMatch = !ordered ? TASK_RE.exec(content) : null;
    if (taskMatch) {
      hasTask = true;
      const checked = taskMatch[1].toLowerCase() === "x" ? " checked" : "";
      items.push(
        `<li class="task-list-item"><input type="checkbox" disabled${checked} /> <span>${renderInline(taskMatch[2])}</span></li>`,
      );
    } else {
      items.push(`<li>${renderInline(content)}</li>`);
    }
    i++;

    while (i < lines.length) {
      const nextLine = lines[i];
      const nextUl = UL_ITEM_RE.exec(nextLine);
      const nextOl = OL_ITEM_RE.exec(nextLine);
      if (nextUl && nextUl[1].length > baseIndent) {
        const nested = parseList(lines, i, nextUl[1].length, false);
        items[items.length - 1] = items[items.length - 1].replace(/<\/li>$/, `${nested.html}</li>`);
        i = nested.nextIndex;
        continue;
      }
      if (nextOl && nextOl[1].length > baseIndent) {
        const nested = parseList(lines, i, nextOl[1].length, true);
        items[items.length - 1] = items[items.length - 1].replace(/<\/li>$/, `${nested.html}</li>`);
        i = nested.nextIndex;
        continue;
      }
      break;
    }
  }

  const tag = ordered ? "ol" : "ul";
  const classAttr = hasTask ? ' class="task-list"' : "";
  return {
    html: `<${tag}${classAttr}>${items.join("")}</${tag}>`,
    nextIndex: i,
  };
}

export function renderMarkdownToHtml(markdown: string): string {
  if (!markdown.trim()) return "";
  const lines = markdown.replace(/\r\n?/g, "\n").split("\n");
  const blocks: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i++;
      continue;
    }

    const fenceMatch = FENCE_RE.exec(line);
    if (fenceMatch) {
      const fenceMarker = fenceMatch[1];
      const fenceChar = fenceMarker[0];
      const langRaw = fenceMatch[2].trim().split(/\s+/)[0] ?? "";
      const codeLines: string[] = [];
      i++;
      const closeRe = new RegExp(`^ {0,3}${fenceChar}{${fenceMarker.length},}\\s*$`);
      while (i < lines.length && !closeRe.test(lines[i])) {
        codeLines.push(lines[i]);
        i++;
      }
      if (i < lines.length) i++;

      const codeText = codeLines.join("\n");
      const normLang = langRaw ? normalizeHighlightLanguage(langRaw) : "";
      const highlighted = normLang ? highlightCode(codeText, normLang) : escapeHtml(codeText);
      const langClass = normLang ? ` class="language-${escapeHtml(normLang)}"` : "";
      blocks.push(`<pre class="md-code-block"><code${langClass}>${highlighted}</code></pre>`);
      continue;
    }

    const headingMatch = HEADING_RE.exec(line);
    if (headingMatch) {
      const level = headingMatch[1].length;
      blocks.push(`<h${level}>${renderInline(headingMatch[2])}</h${level}>`);
      i++;
      continue;
    }

    if (HR_RE.test(line)) {
      blocks.push("<hr />");
      i++;
      continue;
    }

    if (BLOCKQUOTE_RE.test(line)) {
      const quoteLines: string[] = [];
      while (i < lines.length) {
        const qm = BLOCKQUOTE_RE.exec(lines[i]);
        if (!qm) break;
        quoteLines.push(qm[1]);
        i++;
      }
      blocks.push(`<blockquote>${renderMarkdownToHtml(quoteLines.join("\n"))}</blockquote>`);
      continue;
    }

    if (isTableStart(lines, i)) {
      const headers = splitTableRow(lines[i]);
      const aligns = parseTableAlignments(lines[i + 1]);
      i += 2;
      const bodyRows: string[] = [];
      while (i < lines.length && lines[i].trim() && lines[i].includes("|")) {
        const cells = splitTableRow(lines[i]);
        const rowHtml = headers
          .map((_, colIdx) => `<td${alignAttr(aligns[colIdx])}>${renderInline(cells[colIdx] ?? "")}</td>`)
          .join("");
        bodyRows.push(`<tr>${rowHtml}</tr>`);
        i++;
      }
      const headHtml = headers
        .map((h, colIdx) => `<th${alignAttr(aligns[colIdx])}>${renderInline(h)}</th>`)
        .join("");
      const tbody = bodyRows.length > 0 ? `<tbody>${bodyRows.join("")}</tbody>` : "";
      blocks.push(`<table><thead><tr>${headHtml}</tr></thead>${tbody}</table>`);
      continue;
    }

    const ulMatch = UL_ITEM_RE.exec(line);
    if (ulMatch) {
      const list = parseList(lines, i, ulMatch[1].length, false);
      blocks.push(list.html);
      i = list.nextIndex;
      continue;
    }

    const olMatch = OL_ITEM_RE.exec(line);
    if (olMatch) {
      const list = parseList(lines, i, olMatch[1].length, true);
      blocks.push(list.html);
      i = list.nextIndex;
      continue;
    }

    const paraLines: string[] = [];
    while (i < lines.length && !isBlockStart(lines, i)) {
      paraLines.push(lines[i]);
      i++;
    }
    const paraHtml = paraLines
      .map((pl, idx) => {
        if (idx < paraLines.length - 1 && (pl.endsWith("  ") || pl.endsWith("\\"))) {
          return `${renderInline(pl.replace(/(?: {2}|\\)$/, ""))}<br />`;
        }
        return renderInline(pl.trimEnd());
      })
      .join(" ");
    blocks.push(`<p>${paraHtml}</p>`);
  }

  return blocks.join("\n");
}
