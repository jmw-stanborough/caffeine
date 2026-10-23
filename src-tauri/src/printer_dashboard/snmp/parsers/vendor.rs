const MIN_VENDOR_NAME_LEN: usize = 2;

fn is_generic_printer_word(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "printer"
            | "printers"
            | "series"
            | "color"
            | "colour"
            | "laser"
            | "inkjet"
            | "thermal"
            | "network"
            | "mfp"
            | "mono"
            | "monochrome"
            | "duplex"
            | "generic"
            | "unknown"
            | "office"
            | "copy"
            | "copier"
            | "scanner"
            | "ethernet"
    )
}

pub fn clean_vendor_name(raw: &str) -> Option<String> {
    let primary = raw
        .split([',', ';', '\n', '/'])
        .next()
        .unwrap_or(raw)
        .trim();
    if primary.is_empty() {
        return None;
    }

    let mut words: Vec<&str> = primary.split_whitespace().collect();
    while words.len() > 1 {
        let last_clean = words
            .last()
            .unwrap()
            .trim_matches(|c: char| !c.is_ascii_alphanumeric())
            .to_lowercase();
        if matches!(
            last_clean.as_str(),
            "corporation"
                | "corp"
                | "inc"
                | "ltd"
                | "co"
                | "industries"
                | "company"
                | "international"
                | "electronics"
                | "group"
                | "systems"
        ) {
            words.pop();
        } else {
            break;
        }
    }

    let cleaned = words.join(" ");
    let trimmed = cleaned.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    if trimmed.len() >= MIN_VENDOR_NAME_LEN
        && trimmed.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && !is_generic_printer_word(&trimmed.to_lowercase())
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn extract_printer_make(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // IEEE 1284 device ID strings embed manufacturer and model inside semicolon-delimited
    // key-value pairs (e.g. "MFG:Brother Industries, Ltd.;MDL:HL-L3240CDW;"), so inspect
    // structured segments before falling back to leading-word extraction.
    for part in trimmed.split([';', ',', '\n']) {
        let seg = part.trim();
        if let Some((k, v)) = seg.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim();
            if key == "mfg" || key == "manufacturer" {
                if let Some(vendor) = clean_vendor_name(val) {
                    return Some(vendor);
                }
            } else if (key == "pid" || key == "mdl" || key == "model")
                && let Some(make) = extract_printer_make(val)
            {
                return Some(make);
            }
        }
    }

    let first_clause = trimmed
        .split([';', ',', '\n', '/'])
        .next()
        .unwrap_or(trimmed)
        .trim();
    let tokens: Vec<&str> = first_clause.split_whitespace().collect();
    if tokens.len() >= MIN_VENDOR_NAME_LEN {
        let candidate = tokens[0].trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        let valid_chars = candidate.len() >= MIN_VENDOR_NAME_LEN
            && candidate.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && candidate.chars().last().is_some_and(|c| c.is_ascii_alphabetic())
            && candidate.chars().all(|c| c.is_ascii_alphabetic() || c == '-');
        if valid_chars && !is_generic_printer_word(&candidate.to_lowercase()) {
            return Some(candidate.to_string());
        }
    }

    None
}

pub fn infer_capabilities_from_model(
    model: Option<&str>,
    description: Option<&str>,
) -> (Option<bool>, Option<bool>) {
    let combined = format!(
        "{} {}",
        model.unwrap_or(""),
        description.unwrap_or("")
    )
    .to_lowercase();

    let is_color = if combined.contains("color")
        || combined.contains("colour")
        || combined.contains("cdw")
        || combined.contains("cdn")
    {
        Some(true)
    } else if combined.contains("mono") || combined.contains("monochrome") {
        Some(false)
    } else {
        None
    };

    let is_duplex = if combined.contains("duplex") {
        Some(true)
    } else if combined.contains("simplex") {
        Some(false)
    } else {
        let mut has_duplex = false;
        for token in combined.split(|c: char| !c.is_alphanumeric() && c != '-') {
            let t = token.trim();
            if t.len() >= 2 {
                if t.ends_with("dn")
                    || t.ends_with("dw")
                    || t.ends_with("cdw")
                    || t.ends_with("fdw")
                    || t.ends_with("dne")
                    || t.ends_with("dni")
                    || t.ends_with("dwt")
                    || t.ends_with("-d")
                {
                    has_duplex = true;
                    break;
                }
            }
        }
        if has_duplex {
            Some(true)
        } else {
            None
        }
    };

    (is_color, is_duplex)
}
