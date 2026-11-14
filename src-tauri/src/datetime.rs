// Howard Hinnant's civil-from-days algorithm avoids external date crates.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m as u32, d as u32)
}

pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = y - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = m as u64;
    let d = d as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + (doe as i64) - 719_468
}

pub fn format_epoch_secs_as_iso(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem_secs = secs % 86_400;
    let hours = rem_secs / 3_600;
    let minutes = (rem_secs % 3_600) / 60;
    let seconds = rem_secs % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

pub fn current_utc_timestamp() -> Result<String, String> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock is before Unix epoch: {e}"))?
        .as_secs();

    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;

    let (year, m, d) = civil_from_days(days);
    Ok(format!("{year:04}-{m:02}-{d:02} {hour:02}:{min:02}:{sec:02} UTC"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_from_days_round_trip() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(1970, 1, 1), 0);

        let days_leap = days_from_civil(2024, 2, 29);
        assert_eq!(civil_from_days(days_leap), (2024, 2, 29));

        let days_future = days_from_civil(2026, 11, 14);
        assert_eq!(civil_from_days(days_future), (2026, 11, 14));
    }

    #[test]
    fn test_format_epoch_secs_as_iso() {
        assert_eq!(format_epoch_secs_as_iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_epoch_secs_as_iso(1730000000), "2024-10-27T03:33:20Z");
    }

    #[test]
    fn test_current_utc_timestamp() {
        let ts = current_utc_timestamp().unwrap();
        assert!(ts.ends_with(" UTC"));
        assert_eq!(ts.len(), 23);
    }
}
