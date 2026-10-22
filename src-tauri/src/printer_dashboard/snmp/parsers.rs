use std::collections::BTreeMap;

pub const SUPPLY_LEVEL_UNKNOWN: i32 = -2;
pub const SUPPLY_LEVEL_SOME_REMAINING: i32 = -3;
pub const DEFAULT_MAX_CAPACITY: i32 = 100;
const MAX_PERCENT: u8 = 100;

const SUPPLY_TYPE_TONER: i64 = 3;
const SUPPLY_TYPE_WASTE_TONER: i64 = 4;
const SUPPLY_TYPE_INK: i64 = 5;
const SUPPLY_TYPE_INK_CARTRIDGE: i64 = 6;
const SUPPLY_TYPE_WASTE_INK: i64 = 8;
const SUPPLY_TYPE_OPC: i64 = 9;
const SUPPLY_TYPE_FUSER: i64 = 15;
const SUPPLY_TYPE_CLEANER_UNIT: i64 = 18;
const SUPPLY_TYPE_TRANSFER_UNIT: i64 = 20;
const SUPPLY_TYPE_TONER_CARTRIDGE: i64 = 21;

const BROTHER_ENTRY_LEN: usize = 7;
const BROTHER_TYPE_MARKER: u8 = 0x01;
const BROTHER_VALUE_LEN_MARKER: u8 = 0x04;
const BROTHER_SCALE_DIVISOR: u32 = 100;
const BROTHER_EXACT_BLACK: u8 = 0x81;
const BROTHER_EXACT_CYAN: u8 = 0x82;
const BROTHER_EXACT_MAGENTA: u8 = 0x83;
const BROTHER_EXACT_YELLOW: u8 = 0x84;
const BROTHER_SCALED_BLACK: u8 = 0x6F;
const BROTHER_SCALED_CYAN: u8 = 0x70;
const BROTHER_SCALED_MAGENTA: u8 = 0x71;
const BROTHER_SCALED_YELLOW: u8 = 0x72;

const HR_ERROR_JAMMED_MASK: u8 = 0x04;
const HR_ERROR_NO_PAPER_MASK: u8 = 0x40;
const HR_ERROR_INPUT_TRAY_EMPTY_MASK: u8 = 0x04;
const HR_DEVICE_STATUS_DOWN: i64 = 5;
const HR_PRINTER_STATUS_OTHER: i64 = 1;
pub const HR_PRINTER_STATUS_PRINTING: i64 = 4;

const MIN_VENDOR_NAME_LEN: usize = 2;

pub fn infer_supply_color(description: &str, colorant: Option<&str>) -> String {
    let combined = format!("{} {}", colorant.unwrap_or(""), description).to_lowercase();

    if combined.contains("cyan") {
        "cyan".to_string()
    } else if combined.contains("magenta") {
        "magenta".to_string()
    } else if combined.contains("yellow") {
        "yellow".to_string()
    } else if combined.contains("black")
        || combined.contains("noir")
        || combined.contains("schwarz")
        || combined.contains("bk")
    {
        "black".to_string()
    } else if combined.contains("waste") {
        "waste".to_string()
    } else if combined.contains("drum")
        || combined.contains("imaging")
        || combined.contains("photoconductor")
        || combined.contains("opc")
    {
        "drum".to_string()
    } else if combined.contains("fuser")
        || combined.contains("transfer")
        || combined.contains("belt")
        || combined.contains("kit")
    {
        "maintenance".to_string()
    } else {
        "black".to_string()
    }
}

pub fn supply_type_label(type_code: Option<i64>, description: &str) -> String {
    let desc_lower = description.to_lowercase();
    match type_code {
        Some(SUPPLY_TYPE_TONER | SUPPLY_TYPE_TONER_CARTRIDGE) => "toner".to_string(),
        Some(SUPPLY_TYPE_WASTE_TONER) => "waste_toner".to_string(),
        Some(SUPPLY_TYPE_INK | SUPPLY_TYPE_INK_CARTRIDGE) => "ink".to_string(),
        Some(SUPPLY_TYPE_WASTE_INK) => "waste_ink".to_string(),
        Some(SUPPLY_TYPE_OPC) => "drum".to_string(),
        Some(SUPPLY_TYPE_FUSER | SUPPLY_TYPE_CLEANER_UNIT | SUPPLY_TYPE_TRANSFER_UNIT) => {
            "maintenance".to_string()
        }
        _ => {
            if desc_lower.contains("waste") {
                "waste_toner".to_string()
            } else if desc_lower.contains("drum")
                || desc_lower.contains("imaging")
                || desc_lower.contains("opc")
            {
                "drum".to_string()
            } else if desc_lower.contains("fuser")
                || desc_lower.contains("belt")
                || desc_lower.contains("kit")
                || desc_lower.contains("transfer")
            {
                "maintenance".to_string()
            } else if desc_lower.contains("ink") {
                "ink".to_string()
            } else {
                "toner".to_string()
            }
        }
    }
}

pub fn is_primary_toner_or_ink(supply_type: &str, description: &str) -> bool {
    let desc = description.to_lowercase();
    let is_non_toner_component = [
        "waste",
        "drum",
        "belt",
        "fuser",
        "transfer",
        "collector",
        "maintenance",
        "cleaner",
        "roller",
    ]
    .iter()
    .any(|keyword| desc.contains(keyword));

    if is_non_toner_component {
        return false;
    }
    matches!(
        supply_type,
        "toner" | "ink" | "toner-cartridge" | "ink-cartridge"
    )
}

/// Brother printers often report `-3` ("some remaining") in the standard RFC 3805 supply table
/// while exposing actual percentage values inside a proprietary binary TLV structure.
pub fn parse_brother_maintenance_bytes(data: &[u8]) -> BTreeMap<String, u8> {
    let mut exact_pct = BTreeMap::new();
    let mut scaled_pct = BTreeMap::new();

    let mut i = 0;
    while i + BROTHER_ENTRY_LEN <= data.len() {
        let tag = data[i];
        if data[i + 1] == BROTHER_TYPE_MARKER && data[i + 2] == BROTHER_VALUE_LEN_MARKER {
            let raw = u32::from_be_bytes([data[i + 3], data[i + 4], data[i + 5], data[i + 6]]);
            match tag {
                BROTHER_EXACT_BLACK
                | BROTHER_EXACT_CYAN
                | BROTHER_EXACT_MAGENTA
                | BROTHER_EXACT_YELLOW => {
                    let color = match tag {
                        BROTHER_EXACT_BLACK => "black",
                        BROTHER_EXACT_CYAN => "cyan",
                        BROTHER_EXACT_MAGENTA => "magenta",
                        _ => "yellow",
                    };
                    if raw <= u32::from(MAX_PERCENT) {
                        exact_pct.insert(color.to_string(), raw as u8);
                    }
                }
                BROTHER_SCALED_BLACK
                | BROTHER_SCALED_CYAN
                | BROTHER_SCALED_MAGENTA
                | BROTHER_SCALED_YELLOW => {
                    let color = match tag {
                        BROTHER_SCALED_BLACK => "black",
                        BROTHER_SCALED_CYAN => "cyan",
                        BROTHER_SCALED_MAGENTA => "magenta",
                        _ => "yellow",
                    };
                    let pct = (raw / BROTHER_SCALE_DIVISOR).min(u32::from(MAX_PERCENT)) as u8;
                    scaled_pct.insert(color.to_string(), pct);
                }
                _ => {}
            }
            i += BROTHER_ENTRY_LEN;
        } else {
            i += 1;
        }
    }

    for (color, pct) in scaled_pct {
        exact_pct.entry(color).or_insert(pct);
    }

    exact_pct
}

pub fn compute_percent(level: i32, max_capacity: i32) -> Option<u8> {
    if level >= 0 && max_capacity > 0 {
        let pct = ((f64::from(level) / f64::from(max_capacity)) * f64::from(MAX_PERCENT)).round();
        Some(pct.clamp(0.0, f64::from(MAX_PERCENT)) as u8)
    } else if (0..=i32::from(MAX_PERCENT)).contains(&level) && max_capacity <= 0 {
        Some(level as u8)
    } else if level == SUPPLY_LEVEL_SOME_REMAINING {
        // RFC 3805 defines -3 as "some remaining" without a known numeric percentage.
        None
    } else {
        None
    }
}

pub fn is_paper_jam_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    let has_jam_phrase = [
        "paper jam",
        "media jam",
        "document jam",
        "jammed",
        "misfeed",
        "clear jam",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase));

    if has_jam_phrase {
        return true;
    }

    lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|word| matches!(word, "jam" | "jams" | "jammed" | "misfeed"))
}

pub fn is_out_of_paper_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    if ["toner", "ink", "waste", "output", "bin"]
        .iter()
        .any(|unrelated| lower.contains(unrelated))
    {
        return false;
    }

    let has_paper_phrase = [
        "out of paper",
        "no paper",
        "paper out",
        "paper empty",
        "media empty",
        "media needed",
        "media out",
        "load paper",
        "add paper",
        "insert paper",
        "refill paper",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase));

    if has_paper_phrase {
        return true;
    }

    lower.contains("empty")
        && ["tray", "cassette", "drawer", "feeder", "paper", "media"]
            .iter()
            .any(|container| lower.contains(container))
}

/// Parses RFC 2790 `hrPrinterDetectedErrorState` bitmask (`1.3.6.1.2.1.25.3.5.1.2.1`).
/// Returns `(paper_jam, out_of_paper)`.
pub fn parse_hr_error_state(
    bytes: &[u8],
    hr_device_status: Option<i64>,
    hr_printer_status: Option<i64>,
) -> (bool, bool) {
    let Some(&b0) = bytes.first() else {
        return (false, false);
    };

    let paper_jam = (b0 & HR_ERROR_JAMMED_MASK) != 0;
    let mut out_of_paper = (b0 & HR_ERROR_NO_PAPER_MASK) != 0;

    // Many multi-tray printers assert `inputTrayEmpty` whenever an optional secondary tray is
    // empty even while printing normally from another tray; only treat it as out-of-paper when
    // the device itself reports a halted/down state.
    if !out_of_paper
        && let Some(&b1) = bytes.get(1)
        && (b1 & HR_ERROR_INPUT_TRAY_EMPTY_MASK) != 0
        && (hr_device_status == Some(HR_DEVICE_STATUS_DOWN)
            || hr_printer_status == Some(HR_PRINTER_STATUS_OTHER))
    {
        out_of_paper = true;
    }

    (paper_jam, out_of_paper)
}

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
    // key-value pairs (e.g., "MFG:Brother Industries, Ltd.;MDL:HL-L3240CDW;"), so inspect
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supply_color_and_percent_calculation() {
        assert_eq!(infer_supply_color("Cyan Cartridge TK-5240C", None), "cyan");
        assert_eq!(infer_supply_color("Magenta Toner", None), "magenta");
        assert_eq!(infer_supply_color("Yellow Toner", Some("yellow")), "yellow");
        assert_eq!(infer_supply_color("Black Toner Cartridge", None), "black");
        assert_eq!(infer_supply_color("Waste Toner Box", None), "waste");
        assert_eq!(infer_supply_color("Imaging Drum Unit", None), "drum");

        assert_eq!(compute_percent(75, 100), Some(75));
        assert_eq!(compute_percent(1500, 3000), Some(50));
        assert_eq!(compute_percent(-3, 100), None);
        assert_eq!(compute_percent(-2, -2), None);

        assert!(is_primary_toner_or_ink("toner", "Black Toner Cartridge"));
        assert!(is_primary_toner_or_ink("toner", "Cyan Cartridge HP CF211A"));
        assert!(!is_primary_toner_or_ink("waste_toner", "Waste Toner Box"));
        assert!(!is_primary_toner_or_ink("maintenance", "Belt Unit"));
        assert!(!is_primary_toner_or_ink("drum", "Black Drum Unit"));

        let brother_blob: Vec<u8> = vec![
            0x70, 0x01, 0x04, 0x00, 0x00, 0x22, 0x60, 0x82, 0x01, 0x04, 0x00, 0x00, 0x00, 0x5a,
            0x71, 0x01, 0x04, 0x00, 0x00, 0x22, 0xc4, 0x83, 0x01, 0x04, 0x00, 0x00, 0x00, 0x5a,
            0x72, 0x01, 0x04, 0x00, 0x00, 0x1f, 0xa4, 0x84, 0x01, 0x04, 0x00, 0x00, 0x00, 0x5a,
            0x6f, 0x01, 0x04, 0x00, 0x00, 0x1c, 0x20, 0x81, 0x01, 0x04, 0x00, 0x00, 0x00, 0x50,
        ];
        let parsed_br = parse_brother_maintenance_bytes(&brother_blob);
        assert_eq!(parsed_br.get("black"), Some(&80));
        assert_eq!(parsed_br.get("cyan"), Some(&90));
        assert_eq!(parsed_br.get("magenta"), Some(&90));
        assert_eq!(parsed_br.get("yellow"), Some(&90));
    }

    #[test]
    fn test_paper_jam_and_out_of_paper_detection() {
        assert_eq!(
            parse_hr_error_state(&[0x00, 0x00], Some(2), Some(3)),
            (false, false)
        );
        assert_eq!(
            parse_hr_error_state(&[0x04, 0x00], Some(5), Some(1)),
            (true, false)
        );
        assert_eq!(
            parse_hr_error_state(&[0x40, 0x00], Some(5), Some(1)),
            (false, true)
        );
        assert_eq!(
            parse_hr_error_state(&[0x44, 0x00], Some(5), Some(1)),
            (true, true)
        );
        assert_eq!(
            parse_hr_error_state(&[0x00, 0x04], Some(5), Some(3)),
            (false, true)
        );
        assert_eq!(
            parse_hr_error_state(&[0x00, 0x04], Some(2), Some(1)),
            (false, true)
        );
        assert_eq!(
            parse_hr_error_state(&[0x00, 0x04], Some(2), Some(3)),
            (false, false)
        );

        assert!(is_paper_jam_text("Paper Jam in Tray 1"));
        assert!(is_paper_jam_text("Jam Rear"));
        assert!(is_paper_jam_text("Paper Jammed"));
        assert!(is_paper_jam_text("Misfeed in Duplex Unit"));
        assert!(!is_paper_jam_text("Ready to Print"));
        assert!(!is_paper_jam_text("Sleeping"));

        assert!(is_out_of_paper_text("Out of Paper"));
        assert!(is_out_of_paper_text("No Paper"));
        assert!(is_out_of_paper_text("Paper Out Tray 1"));
        assert!(is_out_of_paper_text("Load Paper in Tray 1"));
        assert!(is_out_of_paper_text("Tray 1 Empty"));
        assert!(is_out_of_paper_text("Cassette 1 Empty"));
        assert!(!is_out_of_paper_text("Toner Empty"));
        assert!(!is_out_of_paper_text("Output Tray Empty"));
        assert!(!is_out_of_paper_text("Ready"));

        assert_eq!(compute_percent(0, 100), Some(0));
        assert_eq!(compute_percent(0, 3000), Some(0));
        assert_eq!(compute_percent(0, -2), Some(0));

        assert_eq!(
            extract_printer_make("HP LaserJet 200 color M251n"),
            Some("HP".to_string())
        );
        assert_eq!(
            extract_printer_make("Brother HL-L3240CDW series"),
            Some("Brother".to_string())
        );
        assert_eq!(
            extract_printer_make("RICOH IM 370"),
            Some("RICOH".to_string())
        );
        assert_eq!(
            extract_printer_make("SHARP MX-4061"),
            Some("SHARP".to_string())
        );
        assert_eq!(
            extract_printer_make(
                "HP ETHERNET MULTI-ENVIRONMENT,SN:CNF1B18468,FN:V050MLH,SVCID:23144,PID:HP LaserJet 200 color M251n"
            ),
            Some("HP".to_string())
        );
        assert_eq!(
            extract_printer_make("MFG:Brother Industries, Ltd.;MDL:HL-L3240CDW series;"),
            Some("Brother".to_string())
        );
        assert_eq!(
            clean_vendor_name("Sharp Corporation"),
            Some("Sharp".to_string())
        );
        assert_eq!(clean_vendor_name("RICOH"), Some("RICOH".to_string()));
    }
}
