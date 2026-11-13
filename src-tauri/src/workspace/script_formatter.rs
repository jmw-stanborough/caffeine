use super::scripts::extension_for_language;

const INDENT_UNIT: &str = "    ";

fn strip_hash_comment_outside_quotes(line: &str, escape_char: char) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        if ch == escape_char && !in_single {
            out.push(ch);
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            out.push(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            out.push(ch);
            continue;
        }
        if ch == '#' && !in_single && !in_double {
            break;
        }
        if !in_single && !in_double {
            out.push(ch);
        }
    }
    out
}

fn count_delimiters(code: &str, open: char, close: char) -> (usize, usize, usize) {
    let trimmed = code.trim();
    let mut leading_close = 0usize;
    for ch in trimmed.chars() {
        if ch == close {
            leading_close += 1;
        } else if !ch.is_whitespace() {
            break;
        }
    }

    let mut opens = 0usize;
    let mut closes = 0usize;
    for ch in trimmed.chars() {
        if ch == open {
            opens += 1;
        } else if ch == close {
            closes += 1;
        }
    }
    (leading_close, opens, closes)
}

fn tokenize_shell_words(code: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in code.chars() {
        if ch == '{' || ch == '}' || ch == ';' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if ch == '{' || ch == '}' {
                tokens.push(ch.to_string());
            }
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn shell_indent_step(line: &str) -> (usize, isize) {
    let code = strip_hash_comment_outside_quotes(line, '\\');
    let tokens = tokenize_shell_words(&code);
    if tokens.is_empty() {
        return (0, 0);
    }

    let (leading_dedent, start_idx, base_after) = if tokens[0] == "else" {
        (1usize, 1usize, 1isize)
    } else if tokens[0] == "elif" {
        (1usize, 1usize, 0isize)
    } else {
        let mut dedent = 0usize;
        for tok in &tokens {
            if matches!(tok.as_str(), "}" | "fi" | "done" | "esac") {
                dedent += 1;
            } else {
                break;
            }
        }
        (dedent, dedent, 0isize)
    };

    let mut after_delta = base_after;
    for tok in &tokens[start_idx..] {
        match tok.as_str() {
            "{" | "then" | "do" | "case" => after_delta += 1,
            "}" | "fi" | "done" | "esac" => after_delta -= 1,
            _ => {}
        }
    }

    (leading_dedent, after_delta)
}

fn is_batch_comment(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("rem ") || lower == "rem" || trimmed.starts_with("::")
}

fn strip_double_quoted_for_batch(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_double = false;
    for ch in line.chars() {
        if ch == '"' {
            in_double = !in_double;
        } else if !in_double {
            out.push(ch);
        }
    }
    out
}

fn apply_block_indentation<F>(lines: &[String], mut step_fn: F) -> Vec<String>
where
    F: FnMut(&str) -> (usize, isize),
{
    let mut out = Vec::with_capacity(lines.len());
    let mut indent: usize = 0;

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            out.push(String::new());
            continue;
        }

        let (leading_dedent, after_delta) = step_fn(trimmed);
        indent = indent.saturating_sub(leading_dedent);
        out.push(format!("{}{}", INDENT_UNIT.repeat(indent), trimmed));

        if after_delta >= 0 {
            indent = indent.saturating_add(after_delta as usize);
        } else {
            indent = indent.saturating_sub(after_delta.unsigned_abs());
        }
    }

    out
}

pub fn format_script_source(language_id: &str, content: &str) -> Result<String, String> {
    let normalized_lang = language_id.trim().to_ascii_lowercase();
    extension_for_language(&normalized_lang)?;

    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = normalized
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect();

    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        return Ok(String::new());
    }

    let formatted_lines = match normalized_lang.as_str() {
        "powershell" => apply_block_indentation(&lines, |line| {
            let code = strip_hash_comment_outside_quotes(line, '`');
            let (leading, opens, closes) = count_delimiters(&code, '{', '}');
            let net = (opens as isize) - (closes as isize) + (leading as isize);
            (leading, net)
        }),
        "sh" | "bash" | "zsh" => apply_block_indentation(&lines, shell_indent_step),
        "cmd" | "bat" => apply_block_indentation(&lines, |line| {
            if is_batch_comment(line) {
                return (0, 0);
            }
            let code = strip_double_quoted_for_batch(line);
            let (leading, opens, closes) = count_delimiters(&code, '(', ')');
            let net = (opens as isize) - (closes as isize) + (leading as isize);
            (leading, net)
        }),
        // Python indentation is semantically significant; preserve leading whitespace.
        "python" => lines,
        other => return Err(format!("Unsupported script language '{other}'")),
    };

    Ok(format!("{}\n", formatted_lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_script_powershell_shell_and_batch() {
        let ps = "if ($true) {\r\nWrite-Host \"a\"   \r\n} else {\r\nWrite-Host \"b\"\r\n}\r\n\r\n";
        let formatted_ps = format_script_source("powershell", ps).unwrap();
        assert_eq!(
            formatted_ps,
            "if ($true) {\n    Write-Host \"a\"\n} else {\n    Write-Host \"b\"\n}\n"
        );

        let ps_windows_path = "if ($p -eq \"C:\\Logs\\\") {\nWrite-Host \"ok\"\n}\n";
        assert_eq!(
            format_script_source("powershell", ps_windows_path).unwrap(),
            "if ($p -eq \"C:\\Logs\\\") {\n    Write-Host \"ok\"\n}\n"
        );

        let sh = "if [ -f foo ]; then\necho yes   \nelif [ -f bar ]; then\necho maybe\nelse\necho no\nfi\nfor i in 1 2; do\necho $i\ndone\n";
        let formatted_sh = format_script_source("sh", sh).unwrap();
        assert_eq!(
            formatted_sh,
            "if [ -f foo ]; then\n    echo yes\nelif [ -f bar ]; then\n    echo maybe\nelse\n    echo no\nfi\nfor i in 1 2; do\n    echo $i\ndone\n"
        );

        let bat = "if exist test.txt (\necho found  \n) else (\necho missing\n)\n";
        let formatted_bat = format_script_source("bat", bat).unwrap();
        assert_eq!(
            formatted_bat,
            "if exist test.txt (\n    echo found\n) else (\n    echo missing\n)\n"
        );

        let py = "def hello():\r\n    print('hi')   \r\n\r\n";
        let formatted_py = format_script_source("python", py).unwrap();
        assert_eq!(formatted_py, "def hello():\n    print('hi')\n");

        let quoted_ps = "if ($true) {\nWrite-Host \"}\" # { ignored }\n}\n";
        assert_eq!(
            format_script_source("powershell", quoted_ps).unwrap(),
            "if ($true) {\n    Write-Host \"}\" # { ignored }\n}\n"
        );

        let multiline_sh = "if cond1\nthen\necho 1\nelif cond2\nthen\necho 2\nfi\n";
        assert_eq!(
            format_script_source("bash", multiline_sh).unwrap(),
            "if cond1\nthen\n    echo 1\nelif cond2\nthen\n    echo 2\nfi\n"
        );
    }
}
