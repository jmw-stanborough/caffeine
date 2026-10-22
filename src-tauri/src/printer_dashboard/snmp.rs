use super::types::TonerSupply;
use std::collections::BTreeMap;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

const SNMP_TIMEOUT: Duration = Duration::from_millis(750);
const IPP_TIMEOUT: Duration = Duration::from_millis(1200);

#[derive(Debug, Default, Clone)]
pub struct PrinterPollDetails {
    pub toners: Vec<TonerSupply>,
    pub detected_model: Option<String>,
    pub serial_number: Option<String>,
    pub page_count: Option<u64>,
    pub printer_state: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
enum SnmpVal {
    Int(i64),
    Str(String),
}

impl SnmpVal {
    fn as_i64(&self) -> Option<i64> {
        match self {
            SnmpVal::Int(v) => Some(*v),
            SnmpVal::Str(s) => s.trim().parse::<i64>().ok(),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            SnmpVal::Str(s) => Some(s.as_str()),
            SnmpVal::Int(_) => None,
        }
    }
}

fn encode_length(len: usize) -> Vec<u8> {
    if len < 128 {
        vec![len as u8]
    } else if len <= 0xFF {
        vec![0x81, len as u8]
    } else {
        vec![0x82, (len >> 8) as u8, (len & 0xFF) as u8]
    }
}

fn encode_tlv(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend(encode_length(payload.len()));
    out.extend_from_slice(payload);
    out
}

fn encode_integer(val: i32) -> Vec<u8> {
    let bytes = val.to_be_bytes();
    let mut start = 0;
    while start < 3 {
        if (bytes[start] == 0x00 && (bytes[start + 1] & 0x80) == 0)
            || (bytes[start] == 0xFF && (bytes[start + 1] & 0x80) != 0)
        {
            start += 1;
        } else {
            break;
        }
    }
    encode_tlv(0x02, &bytes[start..])
}

fn encode_oid(oid: &[u32]) -> Vec<u8> {
    if oid.len() < 2 {
        return encode_tlv(0x06, &[43]);
    }
    let mut body = vec![(oid[0] * 40 + oid[1]) as u8];
    for &sub in &oid[2..] {
        if sub == 0 {
            body.push(0);
        } else {
            let mut temp = Vec::new();
            let mut v = sub;
            while v > 0 {
                temp.push((v & 0x7F) as u8);
                v >>= 7;
            }
            temp.reverse();
            for i in 0..temp.len() - 1 {
                temp[i] |= 0x80;
            }
            body.extend(temp);
        }
    }
    encode_tlv(0x06, &body)
}

fn build_snmp_packet(community: &str, request_id: i32, pdu_tag: u8, oids: &[Vec<u32>]) -> Vec<u8> {
    let mut varbinds_inner = Vec::new();
    for oid in oids {
        let mut vb = encode_oid(oid);
        vb.extend_from_slice(&[0x05, 0x00]); // NULL
        varbinds_inner.extend(encode_tlv(0x30, &vb));
    }
    let varbind_list = encode_tlv(0x30, &varbinds_inner);

    let mut pdu_inner = Vec::new();
    pdu_inner.extend(encode_integer(request_id));
    pdu_inner.extend(encode_integer(0)); // error-status
    pdu_inner.extend(encode_integer(0)); // error-index
    pdu_inner.extend(varbind_list);

    let pdu = encode_tlv(pdu_tag, &pdu_inner);

    let mut msg_inner = Vec::new();
    msg_inner.extend(encode_integer(1)); // version = 1 (SNMPv2c)
    msg_inner.extend(encode_tlv(0x04, community.as_bytes()));
    msg_inner.extend(pdu);

    encode_tlv(0x30, &msg_inner)
}

fn parse_ber_tlv(data: &[u8], pos: &mut usize) -> Option<(u8, usize, usize)> {
    if *pos >= data.len() {
        return None;
    }
    let tag = data[*pos];
    *pos += 1;
    if *pos >= data.len() {
        return None;
    }
    let first_len = data[*pos];
    *pos += 1;
    let len = if (first_len & 0x80) == 0 {
        first_len as usize
    } else {
        let num_bytes = (first_len & 0x7F) as usize;
        if num_bytes == 0 || num_bytes > 4 || *pos + num_bytes > data.len() {
            return None;
        }
        let mut l = 0usize;
        for _ in 0..num_bytes {
            l = (l << 8) | (data[*pos] as usize);
            *pos += 1;
        }
        l
    };
    if *pos + len > data.len() {
        return None;
    }
    let val_start = *pos;
    *pos += len;
    Some((tag, val_start, len))
}

fn decode_oid(bytes: &[u8]) -> Vec<u32> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut oid = vec![(bytes[0] / 40) as u32, (bytes[0] % 40) as u32];
    let mut acc = 0u32;
    for &b in &bytes[1..] {
        acc = (acc << 7) | ((b & 0x7F) as u32);
        if (b & 0x80) == 0 {
            oid.push(acc);
            acc = 0;
        }
    }
    oid
}

fn decode_signed_int(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let mut val = if (bytes[0] & 0x80) != 0 { -1i64 } else { 0i64 };
    for &b in bytes {
        val = (val << 8) | (b as i64);
    }
    val
}

fn decode_unsigned_int(bytes: &[u8]) -> i64 {
    let mut val = 0i64;
    for &b in bytes {
        val = (val << 8) | (b as i64);
    }
    val
}

fn clean_octet_string(bytes: &[u8]) -> String {
    let trimmed: Vec<u8> = bytes
        .iter()
        .copied()
        .take_while(|&b| b != 0)
        .filter(|&b| b >= 0x20 || b == b'\t')
        .collect();
    String::from_utf8_lossy(&trimmed).trim().to_string()
}

fn parse_snmp_response(packet: &[u8]) -> Option<Vec<(Vec<u32>, Option<SnmpVal>)>> {
    let mut pos = 0;
    let (seq_tag, seq_start, seq_len) = parse_ber_tlv(packet, &mut pos)?;
    if seq_tag != 0x30 {
        return None;
    }

    let mut inner = seq_start;
    let seq_end = seq_start + seq_len;

    // version
    let _ = parse_ber_tlv(packet, &mut inner)?;
    // community
    let _ = parse_ber_tlv(packet, &mut inner)?;
    // PDU (GetResponse = 0xA2)
    let (pdu_tag, pdu_start, pdu_len) = parse_ber_tlv(packet, &mut inner)?;
    if pdu_tag != 0xA2 || inner > seq_end {
        return None;
    }

    let mut pdu_pos = pdu_start;
    // request-id
    let _ = parse_ber_tlv(packet, &mut pdu_pos)?;
    // error-status
    let (_, err_start, err_len) = parse_ber_tlv(packet, &mut pdu_pos)?;
    let err_status = decode_signed_int(&packet[err_start..err_start + err_len]);
    if err_status != 0 {
        return None;
    }
    // error-index
    let _ = parse_ber_tlv(packet, &mut pdu_pos)?;
    // varbind-list
    let (vbl_tag, vbl_start, vbl_len) = parse_ber_tlv(packet, &mut pdu_pos)?;
    if vbl_tag != 0x30 || pdu_pos > pdu_start + pdu_len {
        return None;
    }

    let mut vbl_pos = vbl_start;
    let vbl_end = vbl_start + vbl_len;
    let mut results = Vec::new();

    while vbl_pos < vbl_end {
        let (vb_tag, vb_start, vb_len) = parse_ber_tlv(packet, &mut vbl_pos)?;
        if vb_tag != 0x30 {
            break;
        }
        let mut vb_pos = vb_start;
        let (oid_tag, oid_start, oid_len) = parse_ber_tlv(packet, &mut vb_pos)?;
        if oid_tag != 0x06 {
            continue;
        }
        let oid = decode_oid(&packet[oid_start..oid_start + oid_len]);

        let (val_tag, val_start, val_len) = parse_ber_tlv(packet, &mut vb_pos)?;
        if vb_pos > vb_start + vb_len {
            continue;
        }
        let val_slice = &packet[val_start..val_start + val_len];

        let val = match val_tag {
            0x02 => Some(SnmpVal::Int(decode_signed_int(val_slice))),
            0x41 | 0x42 | 0x43 => Some(SnmpVal::Int(decode_unsigned_int(val_slice))),
            0x04 => Some(SnmpVal::Str(clean_octet_string(val_slice))),
            // 0x80 = NoSuchObject, 0x81 = NoSuchInstance, 0x82 = EndOfMibView, 0x05 = Null
            _ => None,
        };

        results.push((oid, val));
    }

    Some(results)
}

async fn snmp_request(
    sock: &UdpSocket,
    community: &str,
    req_id: i32,
    pdu_tag: u8,
    oids: &[Vec<u32>],
) -> Option<Vec<(Vec<u32>, Option<SnmpVal>)>> {
    let pkt = build_snmp_packet(community, req_id, pdu_tag, oids);
    sock.send(&pkt).await.ok()?;
    let mut buf = [0u8; 4096];
    let n = timeout(SNMP_TIMEOUT, sock.recv(&mut buf)).await.ok()?.ok()?;
    parse_snmp_response(&buf[..n])
}

pub fn infer_supply_color(description: &str, colorant: Option<&str>) -> String {
    let combined = format!(
        "{} {}",
        colorant.unwrap_or(""),
        description
    )
    .to_lowercase();

    if combined.contains("cyan") {
        "cyan".to_string()
    } else if combined.contains("magenta") {
        "magenta".to_string()
    } else if combined.contains("yellow") {
        "yellow".to_string()
    } else if combined.contains("black") || combined.contains("noir") || combined.contains("schwarz") || combined.contains("bk") {
        "black".to_string()
    } else if combined.contains("waste") {
        "waste".to_string()
    } else if combined.contains("drum") || combined.contains("imaging") || combined.contains("photoconductor") || combined.contains("opc") {
        "drum".to_string()
    } else if combined.contains("fuser") || combined.contains("transfer") || combined.contains("belt") || combined.contains("kit") {
        "maintenance".to_string()
    } else {
        "black".to_string()
    }
}

fn supply_type_label(type_code: Option<i64>, description: &str) -> String {
    let desc_lower = description.to_lowercase();
    match type_code {
        Some(3) | Some(21) => "toner".to_string(),
        Some(4) => "waste_toner".to_string(),
        Some(5) | Some(6) => "ink".to_string(),
        Some(8) => "waste_ink".to_string(),
        Some(9) => "drum".to_string(),
        Some(15) | Some(18) | Some(20) => "maintenance".to_string(),
        _ => {
            if desc_lower.contains("waste") {
                "waste_toner".to_string()
            } else if desc_lower.contains("drum") || desc_lower.contains("imaging") {
                "drum".to_string()
            } else if desc_lower.contains("fuser") || desc_lower.contains("belt") || desc_lower.contains("kit") {
                "maintenance".to_string()
            } else if desc_lower.contains("ink") {
                "ink".to_string()
            } else {
                "toner".to_string()
            }
        }
    }
}

fn compute_percent(level: i32, max_capacity: i32) -> Option<u8> {
    if level >= 0 && max_capacity > 0 {
        let pct = ((level as f64 / max_capacity as f64) * 100.0).round();
        Some(pct.clamp(0.0, 100.0) as u8)
    } else if (0..=100).contains(&level) && max_capacity <= 0 {
        Some(level as u8)
    } else if level == -3 {
        // RFC 3805: -3 means "some remaining" (often used on non-genuine or basic sensors)
        None
    } else {
        None
    }
}

async fn poll_snmp(host: &str, community: &str) -> Option<PrinterPollDetails> {
    let host_owned = host.to_string();
    let target_addr = tokio::task::spawn_blocking(move || {
        if let Ok(ip) = host_owned.parse::<std::net::IpAddr>() {
            Some(std::net::SocketAddr::new(ip, 161))
        } else {
            format!("{host_owned}:161").to_socket_addrs().ok()?.next()
        }
    })
    .await
    .ok()??;

    let bind_addr = if target_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let sock = UdpSocket::bind(bind_addr).await.ok()?;
    sock.connect(target_addr).await.ok()?;

    let mut details = PrinterPollDetails::default();
    let mut req_id = 100;

    // 1. Poll general info in a single SNMP GET
    let general_oids = vec![
        vec![1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 3, 1],       // hrDeviceDescr.1
        vec![1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 16, 1],      // prtGeneralPrinterName.1
        vec![1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 17, 1],      // prtGeneralSerialNumber.1
        vec![1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 4, 1, 1],   // prtMarkerLifeCount.1.1
        vec![1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 1, 1],       // hrPrinterStatus.1
        vec![1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 1],   // prtConsoleDisplayBufferText.1.1
        vec![1, 3, 6, 1, 2, 1, 1, 1, 0],                 // sysDescr.0
    ];

    let resp = snmp_request(&sock, community, req_id, 0xA0, &general_oids).await?;
    req_id += 1;

    let mut sys_descr = None;
    for (oid, val) in resp {
        let Some(v) = val else { continue };
        if oid == [1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 3, 1] {
            if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
                details.detected_model = Some(s.to_string());
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 16, 1] {
            if details.detected_model.is_none()
                && let Some(s) = v.as_str().filter(|s| !s.is_empty())
            {
                details.detected_model = Some(s.to_string());
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 17, 1] {
            if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
                details.serial_number = Some(s.to_string());
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 4, 1, 1] {
            if let Some(c) = v.as_i64().filter(|&c| c >= 0) {
                details.page_count = Some(c as u64);
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 1, 1] {
            if let Some(st) = v.as_i64() {
                details.printer_state = Some(
                    match st {
                        3 => "Idle",
                        4 => "Printing",
                        5 => "Warmup",
                        1 => "Other",
                        _ => "Ready",
                    }
                    .to_string(),
                );
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 1] {
            if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
                details.status_message = Some(s.to_string());
            }
        } else if oid == [1, 3, 6, 1, 2, 1, 1, 1, 0] {
            if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
                sys_descr = Some(s.to_string());
            }
        }
    }

    if details.detected_model.is_none() {
        details.detected_model = sys_descr.map(|s| {
            s.split([';', ',', '\n'])
                .next()
                .unwrap_or(&s)
                .trim()
                .to_string()
        });
    }

    // 2. Walk prtMarkerSuppliesEntry (1.3.6.1.2.1.43.11.1.1) and prtMarkerColorantValue (1.3.6.1.2.1.43.12.1.1.4)
    // Using parallel column GetNext walks for columns 3(colorantIdx), 5(type), 6(desc), 8(max), 9(level)
    #[derive(Default)]
    struct RawSupply {
        colorant_idx: Option<i64>,
        supply_type: Option<i64>,
        description: Option<String>,
        max_capacity: Option<i32>,
        level: Option<i32>,
    }

    let base_prefix = [1u32, 3, 6, 1, 2, 1, 43, 11, 1, 1];
    let mut current_oids = vec![
        vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 3], // colorantIndex
        vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 5], // type
        vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 6], // description
        vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 8], // maxCapacity
        vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 9], // level
    ];

    let mut supplies_map: BTreeMap<Vec<u32>, RawSupply> = BTreeMap::new();

    for _step in 0..16 {
        let Some(step_resp) = snmp_request(&sock, community, req_id, 0xA1, &current_oids).await
        else {
            break;
        };
        req_id += 1;

        let mut any_in_column = false;
        let mut next_oids = Vec::with_capacity(current_oids.len());

        for (col_idx, (ret_oid, ret_val)) in step_resp.into_iter().enumerate() {
            let expected_col = match col_idx {
                0 => 3,
                1 => 5,
                2 => 6,
                3 => 8,
                _ => 9,
            };

            if ret_oid.len() > 11
                && ret_oid[..10] == base_prefix
                && ret_oid[10] == expected_col
                && let Some(val) = ret_val
            {
                any_in_column = true;
                let instance_key = ret_oid[11..].to_vec();
                let entry = supplies_map.entry(instance_key).or_default();
                match expected_col {
                    3 => entry.colorant_idx = val.as_i64(),
                    5 => entry.supply_type = val.as_i64(),
                    6 => entry.description = val.as_str().map(|s| s.to_string()),
                    8 => entry.max_capacity = val.as_i64().map(|v| v as i32),
                    9 => entry.level = val.as_i64().map(|v| v as i32),
                    _ => {}
                }
            }
            next_oids.push(ret_oid);
        }

        if !any_in_column || next_oids.len() != current_oids.len() {
            break;
        }
        current_oids = next_oids;
    }

    // Also try walking prtMarkerColorantValue (1.3.6.1.2.1.43.12.1.1.4) up to 8 steps
    let colorant_prefix = [1u32, 3, 6, 1, 2, 1, 43, 12, 1, 1, 4];
    let mut colorant_oid = vec![colorant_prefix.to_vec()];
    let mut colorants: BTreeMap<u32, String> = BTreeMap::new();

    for _ in 0..8 {
        let Some(c_resp) = snmp_request(&sock, community, req_id, 0xA1, &colorant_oid).await else {
            break;
        };
        req_id += 1;
        let Some((ret_oid, Some(val))) = c_resp.into_iter().next() else {
            break;
        };
        if ret_oid.len() > 11 && ret_oid[..11] == colorant_prefix {
            if let Some(&last_idx) = ret_oid.last()
                && let Some(s) = val.as_str()
            {
                colorants.insert(last_idx, s.to_string());
            }
            colorant_oid = vec![ret_oid];
        } else {
            break;
        }
    }

    for (_instance, raw) in supplies_map {
        let level = raw.level.unwrap_or(-2);
        let max_capacity = raw.max_capacity.unwrap_or(100);
        let colorant_str = raw
            .colorant_idx
            .and_then(|idx| u32::try_from(idx).ok())
            .and_then(|idx| colorants.get(&idx).map(|s| s.as_str()));

        let desc = raw
            .description
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                colorant_str
                    .map(|c| {
                        let mut ch = c.chars();
                        match ch.next() {
                            Some(f) => format!("{} Toner", f.to_uppercase().collect::<String>() + ch.as_str()),
                            None => "Toner".to_string(),
                        }
                    })
                    .unwrap_or_else(|| "Toner Supply".to_string())
            });

        let color = infer_supply_color(&desc, colorant_str);
        let supply_type = supply_type_label(raw.supply_type, &desc);
        let percent = compute_percent(level, max_capacity);

        details.toners.push(TonerSupply {
            name: desc,
            color,
            level,
            max_capacity,
            percent,
            supply_type,
        });
    }

    Some(details)
}

fn build_ipp_get_attributes_request(host: &str) -> Vec<u8> {
    let uri = format!("ipp://{host}/ipp/print");
    let mut body = Vec::new();
    // IPP 2.0, Operation Get-Printer-Attributes (0x000B), request-id 1
    body.extend_from_slice(&[0x02, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x01]);
    // operation-attributes-tag
    body.push(0x01);

    fn push_attr(buf: &mut Vec<u8>, tag: u8, name: &str, val: &str) {
        buf.push(tag);
        buf.extend_from_slice(&(name.len() as u16).to_be_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(&(val.len() as u16).to_be_bytes());
        buf.extend_from_slice(val.as_bytes());
    }

    push_attr(&mut body, 0x47, "attributes-charset", "utf-8");
    push_attr(&mut body, 0x48, "attributes-natural-language", "en");
    push_attr(&mut body, 0x45, "printer-uri", &uri);

    let requested = [
        "marker-names",
        "marker-colors",
        "marker-types",
        "marker-levels",
        "marker-high-levels",
        "printer-make-and-model",
        "printer-state",
        "printer-state-message",
        "printer-impressions-completed",
    ];
    for (idx, attr) in requested.iter().enumerate() {
        if idx == 0 {
            push_attr(&mut body, 0x44, "requested-attributes", attr);
        } else {
            push_attr(&mut body, 0x44, "", attr);
        }
    }

    // end-of-attributes-tag
    body.push(0x03);
    body
}

fn parse_ipp_response(data: &[u8]) -> Option<PrinterPollDetails> {
    // Strip HTTP headers if present
    let ipp_bytes = if let Some(idx) = data.windows(4).position(|w| w == b"\r\n\r\n") {
        &data[idx + 4..]
    } else {
        data
    };

    if ipp_bytes.len() < 9 {
        return None;
    }

    let mut pos = 8; // skip version(2), status(2), req_id(4)
    let mut current_attr_name = String::new();

    let mut marker_names: Vec<String> = Vec::new();
    let mut marker_colors: Vec<String> = Vec::new();
    let mut marker_types: Vec<String> = Vec::new();
    let mut marker_levels: Vec<i32> = Vec::new();
    let mut marker_highs: Vec<i32> = Vec::new();
    let mut details = PrinterPollDetails::default();

    while pos < ipp_bytes.len() {
        let tag = ipp_bytes[pos];
        pos += 1;
        if tag == 0x03 {
            break; // end-of-attributes
        }
        if tag <= 0x0F {
            // delimiter tag
            continue;
        }
        if pos + 2 > ipp_bytes.len() {
            break;
        }
        let name_len = u16::from_be_bytes([ipp_bytes[pos], ipp_bytes[pos + 1]]) as usize;
        pos += 2;
        if pos + name_len + 2 > ipp_bytes.len() {
            break;
        }
        if name_len > 0 {
            current_attr_name =
                String::from_utf8_lossy(&ipp_bytes[pos..pos + name_len]).to_string();
        }
        pos += name_len;

        let val_len = u16::from_be_bytes([ipp_bytes[pos], ipp_bytes[pos + 1]]) as usize;
        pos += 2;
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
            "marker-levels" if val_len == 4 => {
                marker_levels.push(i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]));
            }
            "marker-high-levels" if val_len == 4 => {
                marker_highs.push(i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]));
            }
            "printer-make-and-model" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if !s.is_empty() {
                    details.detected_model = Some(s);
                }
            }
            "printer-state" if val_len == 4 => {
                let st = i32::from_be_bytes([
                    val_bytes[0],
                    val_bytes[1],
                    val_bytes[2],
                    val_bytes[3],
                ]);
                details.printer_state = Some(
                    match st {
                        3 => "Idle",
                        4 => "Printing",
                        5 => "Stopped",
                        _ => "Ready",
                    }
                    .to_string(),
                );
            }
            "printer-state-message" => {
                let s = String::from_utf8_lossy(val_bytes).trim().to_string();
                if !s.is_empty() {
                    details.status_message = Some(s);
                }
            }
            "printer-impressions-completed" if val_len == 4 => {
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
        let level = marker_levels.get(i).copied().unwrap_or(-2);
        let max_capacity = marker_highs.get(i).copied().unwrap_or(100);
        let supply_type = marker_types
            .get(i)
            .cloned()
            .unwrap_or_else(|| "toner".to_string());
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

    if details.toners.is_empty()
        && details.detected_model.is_none()
        && details.printer_state.is_none()
    {
        None
    } else {
        Some(details)
    }
}

async fn poll_ipp(host: &str) -> Option<PrinterPollDetails> {
    let target = if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:631")
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
        let mut resp = Vec::new();
        let _ = stream.read_to_end(&mut resp).await;
        parse_ipp_response(&resp)
    })
    .await
    .ok()?
}

pub async fn poll_printer_details(host: &str, community: &str) -> PrinterPollDetails {
    let comm = if community.trim().is_empty() {
        "public"
    } else {
        community.trim()
    };

    if let Some(mut snmp_data) = poll_snmp(host, comm).await {
        if snmp_data.toners.is_empty()
            && let Some(ipp_data) = poll_ipp(host).await
        {
            snmp_data.toners = ipp_data.toners;
            if snmp_data.detected_model.is_none() {
                snmp_data.detected_model = ipp_data.detected_model;
            }
            if snmp_data.printer_state.is_none() {
                snmp_data.printer_state = ipp_data.printer_state;
            }
        }
        return snmp_data;
    }

    poll_ipp(host).await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer_dashboard::db::{
        add_printer_log_entry, delete_printer_entry, get_latest_printer_log, get_printers,
        insert_printer, query_printer_logs, update_printer_entry, update_printer_sort_order,
    };
    use crate::printer_dashboard::types::Printer;
    use rusqlite::Connection;

    #[test]
    fn test_snmp_ber_encoding_and_response_parsing() {
        let oid = vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 9, 1, 1];
        let encoded_oid = encode_oid(&oid);
        let mut pos = 0;
        let (tag, start, len) = parse_ber_tlv(&encoded_oid, &mut pos).unwrap();
        assert_eq!(tag, 0x06);
        let decoded = decode_oid(&encoded_oid[start..start + len]);
        assert_eq!(decoded, oid);

        // Build a synthetic SNMP GetResponse packet with integer 85%
        let vb_oid = encode_oid(&oid);
        let vb_val = encode_integer(85);
        let mut vb_inner = Vec::new();
        vb_inner.extend(vb_oid);
        vb_inner.extend(vb_val);
        let vb = encode_tlv(0x30, &vb_inner);
        let vbl = encode_tlv(0x30, &vb);

        let mut pdu_inner = Vec::new();
        pdu_inner.extend(encode_integer(101));
        pdu_inner.extend(encode_integer(0));
        pdu_inner.extend(encode_integer(0));
        pdu_inner.extend(vbl);
        let pdu = encode_tlv(0xA2, &pdu_inner);

        let mut msg_inner = Vec::new();
        msg_inner.extend(encode_integer(1));
        msg_inner.extend(encode_tlv(0x04, b"public"));
        msg_inner.extend(pdu);
        let packet = encode_tlv(0x30, &msg_inner);

        let parsed = parse_snmp_response(&packet).expect("valid snmp response");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, oid);
        assert_eq!(parsed[0].1.as_ref().unwrap().as_i64(), Some(85));
    }

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
    }

    #[test]
    fn test_printer_db_crud_and_logs() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE printers (
                id TEXT PRIMARY KEY,
                host TEXT NOT NULL,
                label TEXT NOT NULL,
                description TEXT,
                model TEXT,
                location TEXT,
                snmp_community TEXT NOT NULL DEFAULT 'public',
                web_url TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE printer_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                printer_id TEXT NOT NULL REFERENCES printers(id) ON DELETE CASCADE,
                event_type TEXT NOT NULL,
                latency_ms INTEGER,
                timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            ",
        )
        .unwrap();

        let p1 = Printer {
            id: "p1".to_string(),
            host: "192.168.1.50".to_string(),
            label: "Reception Printer".to_string(),
            description: Some("Main floor".to_string()),
            model: Some("HP LaserJet".to_string()),
            location: Some("Lobby".to_string()),
            snmp_community: "public".to_string(),
            web_url: None,
        };
        insert_printer(&conn, &p1).unwrap();

        let p2 = Printer {
            id: "p2".to_string(),
            host: "192.168.1.51".to_string(),
            label: "Accounting Printer".to_string(),
            description: None,
            model: None,
            location: Some("Room 302".to_string()),
            snmp_community: "public".to_string(),
            web_url: Some("https://192.168.1.51:8443".to_string()),
        };
        insert_printer(&conn, &p2).unwrap();

        let list = get_printers(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "p1");
        assert_eq!(list[1].id, "p2");

        // Reorder
        update_printer_sort_order(&mut conn, &[p2.clone(), p1.clone()]).unwrap();
        let reordered = get_printers(&conn).unwrap();
        assert_eq!(reordered[0].id, "p2");
        assert_eq!(reordered[1].id, "p1");

        // Update
        let mut p1_updated = p1.clone();
        p1_updated.label = "Updated Reception".to_string();
        update_printer_entry(&conn, "p1", &p1_updated).unwrap();
        let after_update = get_printers(&conn).unwrap();
        assert_eq!(after_update[1].label, "Updated Reception");

        // Logs
        add_printer_log_entry(&conn, "p1", "connected", Some(12)).unwrap();
        let latest = get_latest_printer_log(&conn, "p1").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().0, "connected");

        let logs = query_printer_logs(&conn, "p1", 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].latency_ms, Some(12));

        // Delete cascades logs
        delete_printer_entry(&conn, "p1").unwrap();
        let remaining = get_printers(&conn).unwrap();
        assert_eq!(remaining.len(), 1);
        let logs_after = query_printer_logs(&conn, "p1", 10).unwrap();
        assert!(logs_after.is_empty());
    }
}
