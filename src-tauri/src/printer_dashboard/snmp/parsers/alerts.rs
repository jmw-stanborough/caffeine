const HR_ERROR_JAMMED_MASK: u8 = 0x04;
const HR_ERROR_NO_PAPER_MASK: u8 = 0x40;
const HR_ERROR_INPUT_TRAY_EMPTY_MASK: u8 = 0x04;
const HR_DEVICE_STATUS_DOWN: i64 = 5;
const HR_PRINTER_STATUS_OTHER: i64 = 1;
pub const HR_PRINTER_STATUS_PRINTING: i64 = 4;

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

    // Multi-tray printers assert inputTrayEmpty when an optional secondary tray is empty
    // even while printing normally; only treat as out-of-paper if the device is halted or down.
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
