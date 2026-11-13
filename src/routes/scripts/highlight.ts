export type HighlightTokenType =
  | "comment"
  | "string"
  | "variable"
  | "keyword"
  | "number"
  | "operator";

interface TokenRule {
  type: HighlightTokenType;
  pattern: RegExp;
}

interface LanguageSpec {
  caseInsensitiveKeywords?: boolean;
  keywords: ReadonlySet<string>;
  rules: TokenRule[];
}

const POWERSHELL_KEYWORDS = new Set([
  "begin",
  "break",
  "catch",
  "class",
  "continue",
  "data",
  "do",
  "dynamicparam",
  "else",
  "elseif",
  "end",
  "enum",
  "exit",
  "filter",
  "finally",
  "for",
  "foreach",
  "from",
  "function",
  "if",
  "in",
  "param",
  "process",
  "return",
  "switch",
  "throw",
  "trap",
  "try",
  "until",
  "using",
  "while",
  "workflow",
]);

const CMD_KEYWORDS = new Set([
  "call",
  "cd",
  "chdir",
  "cls",
  "copy",
  "defined",
  "del",
  "dir",
  "do",
  "echo",
  "else",
  "endlocal",
  "equ",
  "erase",
  "errorlevel",
  "exist",
  "exit",
  "for",
  "geq",
  "goto",
  "gtr",
  "if",
  "in",
  "leq",
  "lss",
  "md",
  "mkdir",
  "move",
  "neq",
  "not",
  "off",
  "on",
  "pause",
  "popd",
  "pushd",
  "rd",
  "ren",
  "rename",
  "rmdir",
  "robocopy",
  "set",
  "setlocal",
  "shift",
  "start",
  "title",
  "type",
  "xcopy",
]);

const SHELL_KEYWORDS = new Set([
  "break",
  "case",
  "cd",
  "continue",
  "coproc",
  "declare",
  "do",
  "done",
  "echo",
  "elif",
  "else",
  "esac",
  "eval",
  "exec",
  "exit",
  "export",
  "false",
  "fi",
  "for",
  "function",
  "if",
  "in",
  "local",
  "printf",
  "read",
  "readonly",
  "return",
  "select",
  "shift",
  "source",
  "test",
  "then",
  "time",
  "trap",
  "true",
  "typeset",
  "unset",
  "until",
  "while",
]);

const PYTHON_KEYWORDS = new Set([
  "False",
  "None",
  "True",
  "and",
  "as",
  "assert",
  "async",
  "await",
  "break",
  "case",
  "class",
  "continue",
  "def",
  "del",
  "elif",
  "else",
  "except",
  "finally",
  "for",
  "from",
  "global",
  "if",
  "import",
  "in",
  "is",
  "lambda",
  "match",
  "nonlocal",
  "not",
  "or",
  "pass",
  "raise",
  "return",
  "try",
  "while",
  "with",
  "yield",
]);

const POWERSHELL_SPEC: LanguageSpec = {
  caseInsensitiveKeywords: true,
  keywords: POWERSHELL_KEYWORDS,
  rules: [
    { type: "comment", pattern: /<#[\s\S]*?#>/y },
    { type: "comment", pattern: /#[^\r\n]*/y },
    { type: "string", pattern: /@"[\r\n][\s\S]*?[\r\n]"@/y },
    { type: "string", pattern: /@'[\r\n][\s\S]*?[\r\n]'@/y },
    { type: "string", pattern: /"(?:`[\s\S]|[^"\\`]|\\.)*"/y },
    { type: "string", pattern: /'(?:''|[^'])*'/y },
    { type: "variable", pattern: /\$\{[^}]+\}|\$(?:[A-Za-z_][A-Za-z0-9_:]*|[_?$^])/y },
    { type: "number", pattern: /\b0x[0-9a-fA-F]+\b|\b\d+(?:\.\d+)?(?:[kmgtp]b)?\b/iy },
    {
      type: "operator",
      pattern:
        /-(?:eq|ne|gt|ge|lt|le|like|notlike|match|notmatch|contains|notcontains|in|notin|replace|split|join|and|or|xor|not|band|bor|is|as)\b|&&|\|\||\|>|\+=|-=|\*=|\/=|%=|\+\+|--|[|=+\-*/%!<>]/iy,
    },
  ],
};

const CMD_SPEC: LanguageSpec = {
  caseInsensitiveKeywords: true,
  keywords: CMD_KEYWORDS,
  rules: [
    { type: "comment", pattern: /(?:^|(?<=[\r\n]))[ \t]*(?:::[^\r\n]*|rem(?=[ \t\r\n]|$)[^\r\n]*)/iy },
    { type: "comment", pattern: /(?<=[ \t&|])rem(?=[ \t\r\n]|$)[^\r\n]*/iy },
    { type: "keyword", pattern: /@echo\b/iy },
    { type: "string", pattern: /"[^"\r\n]*"/y },
    { type: "variable", pattern: /%~?[A-Za-z0-9_*-]+%|![A-Za-z0-9_.-]+!|%~[fdpnxsatz]*[0-9]|%[0-9*]/y },
    { type: "number", pattern: /\b\d+\b/y },
    { type: "operator", pattern: /&&|\|\||>>|==|[&|<>()=@]/y },
  ],
};

const SHELL_SPEC: LanguageSpec = {
  keywords: SHELL_KEYWORDS,
  rules: [
    { type: "comment", pattern: /(?:^|(?<=[\s;|&()]))#[^\r\n]*/y },
    { type: "string", pattern: /\$'(?:\\[\s\S]|[^'\\])*'/y },
    { type: "string", pattern: /"(?:\\[\s\S]|[^"\\])*"/y },
    { type: "string", pattern: /'[^']*'/y },
    { type: "variable", pattern: /\$\{[^}]+\}|\$(?:[A-Za-z_][A-Za-z0-9_]*|[0-9?$@*#!_-])/y },
    { type: "number", pattern: /\b0x[0-9a-fA-F]+\b|\b\d+(?:\.\d+)?\b/y },
    { type: "operator", pattern: /&&|\|\||;;|<<-?|>>|>&|<&|\|&|==|!=|=~|[=|&;<>!]/y },
  ],
};

const PYTHON_SPEC: LanguageSpec = {
  keywords: PYTHON_KEYWORDS,
  rules: [
    { type: "comment", pattern: /#[^\r\n]*/y },
    { type: "string", pattern: /[fFbBrRuU]{0,2}"""[\s\S]*?"""/y },
    { type: "string", pattern: /[fFbBrRuU]{0,2}'''[\s\S]*?'''/y },
    { type: "string", pattern: /[fFbBrRuU]{0,2}"(?:\\[\s\S]|[^"\\\r\n])*"/y },
    { type: "string", pattern: /[fFbBrRuU]{0,2}'(?:\\[\s\S]|[^'\\\r\n])*'/y },
    { type: "variable", pattern: /\b(?:self|cls)\b|@[A-Za-z_][A-Za-z0-9_.]*/y },
    {
      type: "number",
      pattern: /\b0[xXoObB][0-9a-fA-F_]+\b|\b\d[\d_]*(?:\.[\d_]+)?(?:[eE][+-]?\d[\d_]*)?j?\b/y,
    },
    {
      type: "operator",
      pattern: /\/\/=|\*\*=|<<=|>>=|:=|->|==|!=|<=|>=|\+=|-=|\*=|\/=|%=|&=|\|=|\^=|\/\/|\*\*|<<|>>|[+\-*/%=<>&|^~!]/y,
    },
  ],
};

const LANGUAGE_MAP: Record<string, LanguageSpec> = {
  powershell: POWERSHELL_SPEC,
  ps1: POWERSHELL_SPEC,
  pwsh: POWERSHELL_SPEC,
  cmd: CMD_SPEC,
  bat: CMD_SPEC,
  batch: CMD_SPEC,
  sh: SHELL_SPEC,
  bash: SHELL_SPEC,
  zsh: SHELL_SPEC,
  shell: SHELL_SPEC,
  python: PYTHON_SPEC,
  py: PYTHON_SPEC,
};

export function normalizeHighlightLanguage(languageId: string): string {
  const normalized = languageId.trim().toLowerCase();
  switch (normalized) {
    case "ps1":
    case "pwsh":
    case "powershell":
      return "powershell";
    case "cmd":
      return "cmd";
    case "bat":
    case "batch":
      return "bat";
    case "sh":
    case "shell":
      return "sh";
    case "bash":
      return "bash";
    case "zsh":
      return "zsh";
    case "py":
    case "python":
      return "python";
    default:
      return normalized;
  }
}

export function escapeHtml(input: string): string {
  return input
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function wrapToken(type: HighlightTokenType, raw: string): string {
  return `<span class="hl-${type}">${escapeHtml(raw)}</span>`;
}

function isWordChar(ch: string): boolean {
  return /[A-Za-z0-9_]/.test(ch);
}

export function highlightCode(code: string, languageId: string): string {
  if (!code) return "";
  const spec = LANGUAGE_MAP[normalizeHighlightLanguage(languageId)];
  if (!spec) {
    return escapeHtml(code);
  }

  let out = "";
  let pos = 0;
  let plainStart = 0;

  while (pos < code.length) {
    let matchedType: HighlightTokenType | null = null;
    let matchedText = "";

    for (const rule of spec.rules) {
      rule.pattern.lastIndex = pos;
      const match = rule.pattern.exec(code);
      if (match) {
        matchedType = rule.type;
        matchedText = match[0];
        break;
      }
    }

    if (!matchedType) {
      const ch = code[pos];
      if (isWordChar(ch) && (pos === 0 || (!isWordChar(code[pos - 1]) && code[pos - 1] !== "-"))) {
        let end = pos + 1;
        while (end < code.length && isWordChar(code[end])) {
          end++;
        }
        const word = code.slice(pos, end);
        const lookup = spec.caseInsensitiveKeywords ? word.toLowerCase() : word;
        if (spec.keywords.has(lookup)) {
          if (pos > plainStart) {
            out += escapeHtml(code.slice(plainStart, pos));
          }
          out += wrapToken("keyword", word);
          pos = end;
          plainStart = end;
          continue;
        }
      }
      pos++;
      continue;
    }

    if (pos > plainStart) {
      out += escapeHtml(code.slice(plainStart, pos));
    }
    out += wrapToken(matchedType, matchedText);
    pos += matchedText.length;
    plainStart = pos;
  }

  if (plainStart < code.length) {
    out += escapeHtml(code.slice(plainStart));
  }

  return out;
}
