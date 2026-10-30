fn contains_unsafe_chars(s: &str) -> bool {
    s.chars()
        .any(|c| c.is_whitespace() || matches!(c, ';' | '&' | '|' | '`' | '$' | '\'' | '"'))
}

pub fn sanitize_host(host: &str) -> Result<String, String> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    if contains_unsafe_chars(trimmed) {
        return Err("Host contains invalid or unsafe characters".to_string());
    }

    Ok(trimmed.to_string())
}

pub fn sanitize_username(username: Option<&str>) -> Result<Option<String>, String> {
    match username {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if contains_unsafe_chars(trimmed) {
                return Err("Username contains invalid or unsafe characters".to_string());
            }
            Ok(Some(trimmed.to_string()))
        }
    }
}

pub fn resolve_port(port: Option<u16>, default_port: u16) -> Result<u16, String> {
    match port {
        Some(0) => Err("Port number cannot be 0".to_string()),
        Some(p) => Ok(p),
        None => Ok(default_port),
    }
}
