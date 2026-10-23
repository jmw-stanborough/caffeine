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

/// Brother printers report -3 ("some remaining") in the standard RFC 3805 supply table
/// while exposing actual percentages inside a proprietary binary TLV structure.
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
