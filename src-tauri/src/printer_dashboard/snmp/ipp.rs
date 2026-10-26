use super::PrinterPollDetails;
use super::parsers::{
    DEFAULT_MAX_CAPACITY, HR_PRINTER_STATUS_PRINTING, SUPPLY_LEVEL_UNKNOWN, compute_percent,
    extract_printer_make, infer_supply_color, is_out_of_paper_text, is_paper_jam_text,
    is_primary_toner_or_ink,
};
use crate::printer_dashboard::types::TonerSupply;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const IPP_TIMEOUT: Duration = Duration::from_millis(1200);
const DEFAULT_IPP_PORT: u16 = 631;

const IPP_HEADER_GET_PRINTER_ATTRIBUTES: [u8; 8] = [0x02, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x01];
const IPP_TAG_OPERATION_ATTRIBUTES: u8 = 0x01;
const IPP_TAG_END_OF_ATTRIBUTES: u8 = 0x03;
const IPP_MAX_DELIMITER_TAG: u8 = 0x0F;
const IPP_TAG_KEYWORD: u8 = 0x44;
const IPP_TAG_URI: u8 = 0x45;
const IPP_TAG_CHARSET: u8 = 0x47;
const IPP_TAG_NATURAL_LANGUAGE: u8 = 0x48;

const IPP_HEADER_LEN: usize = 8;
const MIN_IPP_RESPONSE_LEN: usize = 9;
const IPP_LENGTH_FIELD_SIZE: usize = 2;
const IPP_INT32_VALUE_LEN: usize = 4;
const HTTP_HEADER_DELIMITER: &[u8] = b"\r\n\r\n";

fn build_ipp_get_attributes_request(host: &str) -> Vec<u8> {
    let uri = format!("ipp://{host}/ipp/print");
    let mut body = Vec::new();
    body.extend_from_slice(&IPP_HEADER_GET_PRINTER_ATTRIBUTES);
    body.push(IPP_TAG_OPERATION_ATTRIBUTES);

    fn push_attr(buf: &mut Vec<u8>, tag: u8, name: &str, val: &str) {
        buf.push(tag);
        buf.extend_from_slice(&(name.len() as u16).to_be_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(&(val.len() as u16).to_be_bytes());
        buf.extend_from_slice(val.as_bytes());
    }

    push_attr(&mut body, IPP_TAG_CHARSET, "attributes-charset", "utf-8");
    push_attr(
        &mut body,
        IPP_TAG_NATURAL_LANGUAGE,
        "attributes-natural-language",
        "en",
    );
    push_attr(&mut body, IPP_TAG_URI, "printer-uri", &uri);

    let requested = [
        "marker-names",
        "marker-colors",
        "marker-types",
        "marker-levels",
        "marker-high-levels",
        "printer-device-id",
        "printer-make-and-model",
        "printer-location",
        "printer-state",
        "printer-state-reasons",
        "printer-state-message",
        "printer-impressions-completed",
        "color-supported",
        "print-color-mode-supported",
        "sides-supported",
    ];
    for (idx, attr) in requested.iter().enumerate() {
        let attr_name = if idx == 0 { "requested-attributes" } else { "" };
        push_attr(&mut body, IPP_TAG_KEYWORD, attr_name, attr);
    }

    body.push(IPP_TAG_END_OF_ATTRIBUTES);
    body
}

fn parse_ipp_response(data: &[u8]) -> Option<PrinterPollDetails> {
    let ipp_bytes = if let Some(idx) = data
        .windows(HTTP_HEADER_DELIMITER.len())
        .position(|w| w == HTTP_HEADER_DELIMITER)
    {
        &data[idx + HTTP_HEADER_DELIMITER.len()..]
    } else {
        data
    };

    if ipp_bytes.len() < MIN_IPP_RESPONSE_LEN {
        return None;
    }

    let mut pos = IPP_HEADER_LEN;
    let mut current_attr_name = String::new();

    let mut marker_names: Vec<String> = Vec::new();
    let mut marker_colors: Vec<String> = Vec::new();
    let mut marker_types: Vec<String> = Vec::new();
    let mut marker_levels: Vec<i32> = Vec::new();
    let mut marker_highs: Vec<i32> = Vec::new();
    let mut has_ipp_color_mode = false;
    let mut has_ipp_mono_mode = false;
    let mut has_ipp_duplex = false;
    let mut has_ipp_simplex = false;
    let mut details = PrinterPollDetails::default();

    while pos < ipp_bytes.len() {
        let tag = ipp_bytes[pos];
        pos += 1;
        if tag == IPP_TAG_END_OF_ATTRIBUTES {
            break;
        }
        if tag <= IPP_MAX_DELIMITER_TAG {
            continue;
        }
        if pos + IPP_LENGTH_FIELD_SIZE > ipp_bytes.len() {
            break;
        }
        let name_len = u16::from_be_bytes([ipp_bytes[pos], ipp_bytes[pos + 1]]) as usize;
        pos += IPP_LENGTH_FIELD_SIZE;
        if pos + name_len + IPP_LENGTH_FIELD_SIZE > ipp_bytes.len() {
            break;
        }
        if name_len > 0 {
            current_attr_name =
                String::from_utf8_lossy(&ipp_bytes[pos..pos + name_len]).to_string();
        }
        pos += name_len;

        let val_len = u16::from_be_bytes([ipp_bytes[pos], ipp_bytes[pos + 1]]) as usize;
        pos += IPP_LENGTH_FIELD_SIZE;
        if pos + val_len > ipp_bytes.len() {
            break;
        }
        let val_bytes = &ipp_bytes[pos..pos + val_len];
        pos += val_len;

        match current_attr_name.as_str() {
            "marker-names" => {
                marker_names.push(String::from_utf8_lossy(val_bytes).trim().to_string());
            }
            "marker-colors" => {
                marker_colors.push(String::from_utf8_lossy(val_bytes).trim().to_string());
            }
            "marker-types" => {
                marker_types.push(String::from_utf8_lossy(val_bytes).trim().to_string());
            }
            "marker-levels" if val_len == IPP_INT32_VALUE_LEN => {
                marker_levels.push(i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]));
            }
            "marker-high-levels" if val_len == IPP_INT32_VALUE_LEN => {
                marker_highs.push(i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]));
            }
            "printer-device-id" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if details.detected_make.is_none()
                    && let Some(make) = extract_printer_make(&s)
                {
                    details.detected_make = Some(make);
                }
            }
            "printer-make-and-model" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if !s.is_empty() {
                    if details.detected_make.is_none() {
                        details.detected_make = extract_printer_make(&s);
                    }
                    details.detected_model = Some(s);
                }
            }
            "printer-location" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if !s.is_empty() {
                    details.detected_location = Some(s);
                }
            }
            "printer-state" if val_len == IPP_INT32_VALUE_LEN => {
                let st = i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]);
                let state_str = if i64::from(st) == HR_PRINTER_STATUS_PRINTING {
                    "Printing"
                } else {
                    "Idle"
                };
                details.printer_state = Some(state_str.to_string());
            }
            "printer-state-reasons" => {
                let reason = String::from_utf8_lossy(val_bytes).trim().to_lowercase();
                if !reason.is_empty() && reason != "none" && !reason.ends_with("-report") {
                    if reason.contains("media-jam") || reason.split('-').any(|p| p == "jam") {
                        details.paper_jam = true;
                    }
                    if reason.contains("media-empty")
                        || reason.contains("media-needed")
                        || reason.contains("input-tray-empty")
                    {
                        details.out_of_paper = true;
                    }
                }
            }
            "printer-state-message" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if !s.is_empty() {
                    if is_paper_jam_text(&s) {
                        details.paper_jam = true;
                    }
                    if is_out_of_paper_text(&s) {
                        details.out_of_paper = true;
                    }
                    details.status_message = Some(s);
                }
            }
            "printer-impressions-completed" if val_len == IPP_INT32_VALUE_LEN => {
                let c = i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]);
                if c >= 0 {
                    details.page_count = Some(c as u64);
                }
            }
            "color-supported" => {
                if tag == 0x22 && !val_bytes.is_empty() {
                    details.is_color = Some(val_bytes[0] != 0);
                } else {
                    let s = String::from_utf8_lossy(val_bytes).trim().to_lowercase();
                    if s == "true" {
                        details.is_color = Some(true);
                    } else if s == "false" {
                        details.is_color = Some(false);
                    }
                }
            }
            "print-color-mode-supported" => {
                let mode = String::from_utf8_lossy(val_bytes).trim().to_lowercase();
                if mode.contains("color") {
                    has_ipp_color_mode = true;
                }
                if mode.contains("monochrome") || mode.contains("mono") {
                    has_ipp_mono_mode = true;
                }
            }
            "sides-supported" => {
                let side = String::from_utf8_lossy(val_bytes).trim().to_lowercase();
                if side.contains("two-sided") || side.contains("duplex") {
                    has_ipp_duplex = true;
                }
                if side.contains("one-sided") || side.contains("simplex") {
                    has_ipp_simplex = true;
                }
            }
            _ => {}
        }
    }

    let count = marker_names.len().max(marker_levels.len());
    for i in 0..count {
        let name = marker_names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("Supply {}", i + 1));
        let raw_color = marker_colors.get(i).map(|s| s.as_str());
        let color = infer_supply_color(&name, raw_color);
        let level = marker_levels.get(i).copied().unwrap_or(SUPPLY_LEVEL_UNKNOWN);
        let max_capacity = marker_highs
            .get(i)
            .copied()
            .unwrap_or(DEFAULT_MAX_CAPACITY);
        let supply_type = marker_types
            .get(i)
            .cloned()
            .unwrap_or_else(|| "toner".to_string());
        if !is_primary_toner_or_ink(&supply_type, &name) {
            continue;
        }
        let percent = compute_percent(level, max_capacity);

        details.toners.push(TonerSupply {
            name,
            color,
            level,
            max_capacity,
            percent,
            supply_type,
        });
    }

    if details.is_color.is_none() {
        if has_ipp_color_mode {
            details.is_color = Some(true);
        } else if has_ipp_mono_mode {
            details.is_color = Some(false);
        } else if !details.toners.is_empty() {
            let has_cmy_toner = details.toners.iter().any(|t| {
                matches!(t.color.as_str(), "cyan" | "magenta" | "yellow")
            });
            if has_cmy_toner {
                details.is_color = Some(true);
            } else {
                let only_black = details.toners.iter().all(|t| {
                    matches!(t.color.as_str(), "black" | "waste" | "drum" | "maintenance")
                });
                if only_black {
                    details.is_color = Some(false);
                }
            }
        }
    }

    if has_ipp_duplex {
        details.is_duplex = Some(true);
    } else if has_ipp_simplex {
        details.is_duplex = Some(false);
    }

    if details.toners.is_empty()
        && details.detected_make.is_none()
        && details.detected_model.is_none()
        && details.printer_state.is_none()
        && !details.paper_jam
        && !details.out_of_paper
        && details.is_color.is_none()
        && details.is_duplex.is_none()
    {
        None
    } else {
        Some(details)
    }
}

async fn read_http_response<R: AsyncReadExt + Unpin>(
    stream: &mut R,
) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut temp = [0u8; 2048];
    let mut body_expected: Option<usize> = None;
    let mut header_end: Option<usize> = None;

    loop {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = buf
                .windows(HTTP_HEADER_DELIMITER.len())
                .position(|w| w == HTTP_HEADER_DELIMITER)
            {
                let end = pos + HTTP_HEADER_DELIMITER.len();
                header_end = Some(end);

                let headers = String::from_utf8_lossy(&buf[..pos]);
                for line in headers.lines() {
                    if let Some((k, v)) = line.split_once(':') {
                        if k.trim().eq_ignore_ascii_case("content-length") {
                            if let Ok(len) = v.trim().parse::<usize>() {
                                body_expected = Some(len);
                            }
                        }
                    }
                }
            }
        }

        if let (Some(end), Some(expected)) = (header_end, body_expected) {
            if buf.len() >= end + expected {
                break;
            }
        }
    }

    Ok(buf)
}

pub async fn poll_ipp(host: &str) -> Option<PrinterPollDetails> {
    let target = if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:{DEFAULT_IPP_PORT}")
    };

    let mut stream = timeout(IPP_TIMEOUT, TcpStream::connect(&target))
        .await
        .ok()?
        .ok()?;

    let ipp_body = build_ipp_get_attributes_request(host);
    let http_req = format!(
        "POST /ipp/print HTTP/1.1\r\nHost: {}\r\nContent-Type: application/ipp\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        host,
        ipp_body.len()
    );

    timeout(IPP_TIMEOUT, async {
        stream.write_all(http_req.as_bytes()).await.ok()?;
        stream.write_all(&ipp_body).await.ok()?;
        let resp = read_http_response(&mut stream).await.ok()?;
        parse_ipp_response(&resp)
    })
    .await
    .ok()?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_ipp_packet(attributes: &[(u8, &str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
        buf.push(IPP_TAG_OPERATION_ATTRIBUTES);

        for &(tag, name, val) in attributes {
            buf.push(tag);
            buf.extend_from_slice(&(name.len() as u16).to_be_bytes());
            buf.extend_from_slice(name.as_bytes());
            buf.extend_from_slice(&(val.len() as u16).to_be_bytes());
            buf.extend_from_slice(val);
        }

        buf.push(IPP_TAG_END_OF_ATTRIBUTES);
        buf
    }

    #[test]
    fn test_parse_ipp_color_and_duplex_capabilities() {
        let attrs = vec![
            (0x44, "printer-make-and-model", b"Brother HL-L3240CDW" as &[u8]),
            (0x44, "printer-location", b"Room 101" as &[u8]),
            (0x22, "color-supported", &[0x01]),
            (0x44, "sides-supported", b"one-sided"),
            (0x44, "", b"two-sided-long-edge"),
        ];
        let packet = build_test_ipp_packet(&attrs);
        let parsed = parse_ipp_response(&packet).expect("valid response");

        assert_eq!(parsed.detected_model, Some("Brother HL-L3240CDW".to_string()));
        assert_eq!(parsed.detected_location, Some("Room 101".to_string()));
        assert_eq!(parsed.is_color, Some(true));
        assert_eq!(parsed.is_duplex, Some(true));
    }

    #[test]
    fn test_parse_ipp_mono_and_simplex_capabilities() {
        let attrs = vec![
            (0x44, "printer-make-and-model", b"HP LaserJet 1020" as &[u8]),
            (0x22, "color-supported", &[0x00]),
            (0x44, "sides-supported", b"one-sided"),
        ];
        let packet = build_test_ipp_packet(&attrs);
        let parsed = parse_ipp_response(&packet).expect("valid response");

        assert_eq!(parsed.is_color, Some(false));
        assert_eq!(parsed.is_duplex, Some(false));
    }

    #[tokio::test]
    async fn test_read_http_response_content_length() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let _ = server
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: keep-alive\r\n\r\nhello-extra-bytes")
                .await;
            // Intentionally keep server alive without closing
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });

        let resp = read_http_response(&mut client).await.expect("read response");
        let s = String::from_utf8_lossy(&resp);
        assert!(s.ends_with("hello"));
        assert!(!s.contains("extra"));
    }
}
