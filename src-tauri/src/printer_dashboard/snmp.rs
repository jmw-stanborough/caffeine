pub mod ber;
pub mod ipp;
pub mod parsers;

pub use parsers::{
    clean_vendor_name, extract_printer_make, infer_capabilities_from_model, infer_supply_color,
    is_out_of_paper_text, is_paper_jam_text, is_primary_toner_or_ink,
    parse_brother_maintenance_bytes, parse_hr_error_state,
};

use super::types::{TonerSupply, normalize_snmp_community};
use ber::{PDU_GET_NEXT_REQUEST, PDU_GET_REQUEST, snmp_request, snmp_walk_columns};
use ipp::poll_ipp;
use parsers::{
    DEFAULT_MAX_CAPACITY, HR_PRINTER_STATUS_PRINTING, SUPPLY_LEVEL_UNKNOWN, compute_percent,
    supply_type_label,
};
use std::collections::BTreeMap;
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;

const SNMP_PORT: u16 = 161;
const INITIAL_SNMP_REQUEST_ID: i32 = 100;
const MAX_SUPPLY_WALK_STEPS: usize = 16;
const MAX_COLORANT_WALK_STEPS: usize = 8;
const MAX_ALERT_WALK_STEPS: usize = 8;
const MAX_MEDIA_PATH_WALK_STEPS: usize = 8;

const OID_SYS_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 1, 0];
const OID_HR_DEVICE_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 3, 1];
const OID_HR_DEVICE_STATUS: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 2, 1, 5, 1];
const OID_HR_PRINTER_STATUS: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 1, 1];
const OID_HR_PRINTER_ERROR_STATE: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 2, 1];
const OID_PRT_GENERAL_PRINTER_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 16, 1];
const OID_PRT_GENERAL_SERIAL_NUMBER: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 5, 1, 1, 17, 1];
const OID_PRT_INPUT_VENDOR_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 8, 2, 1, 14, 1, 1];
const OID_PRT_MARKER_LIFE_COUNT: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 4, 1, 1];
const OID_PRT_MARKER_PROCESS_COLORANTS: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 6, 1, 1];
const OID_PRT_CONSOLE_LINE_1: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 1];
const OID_PRT_CONSOLE_LINE_2: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 16, 5, 1, 2, 1, 2];
const OID_ENT_PHYSICAL_MFG_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 47, 1, 1, 1, 1, 12, 1];
const OID_BROTHER_MAINTENANCE: &[u32] = &[1, 3, 6, 1, 4, 1, 2435, 2, 3, 9, 4, 2, 1, 5, 5, 8, 0];

const OID_PRT_MARKER_SUPPLIES_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 11, 1, 1];
const SUPPLY_COL_COLORANT_INDEX: u32 = 3;
const SUPPLY_COL_TYPE: u32 = 5;
const SUPPLY_COL_DESCRIPTION: u32 = 6;
const SUPPLY_COL_MAX_CAPACITY: u32 = 8;
const SUPPLY_COL_LEVEL: u32 = 9;
const SUPPLY_WALK_COLUMNS: &[u32] = &[
    SUPPLY_COL_COLORANT_INDEX,
    SUPPLY_COL_TYPE,
    SUPPLY_COL_DESCRIPTION,
    SUPPLY_COL_MAX_CAPACITY,
    SUPPLY_COL_LEVEL,
];

const OID_PRT_MARKER_COLORANT_VALUE: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 12, 1, 1, 4];

const OID_PRT_MEDIA_PATH_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 13, 4, 1];
const MEDIA_PATH_COL_TYPE: u32 = 9;
const MEDIA_PATH_COL_DESCRIPTION: u32 = 10;
const MEDIA_PATH_WALK_COLUMNS: &[u32] = &[MEDIA_PATH_COL_TYPE, MEDIA_PATH_COL_DESCRIPTION];

const OID_PRT_ALERT_ENTRY: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 18, 1, 1];
const ALERT_COL_SEVERITY: u32 = 2;
const ALERT_COL_GROUP: u32 = 5;
const ALERT_COL_CODE: u32 = 7;
const ALERT_COL_DESCRIPTION: u32 = 8;
const ALERT_WALK_COLUMNS: &[u32] = &[
    ALERT_COL_SEVERITY,
    ALERT_COL_GROUP,
    ALERT_COL_CODE,
    ALERT_COL_DESCRIPTION,
];

const ALERT_CODE_JAM: i64 = 8;
const ALERT_CODE_SUBUNIT_EMPTY: i64 = 13;
const ALERT_CODE_INPUT_MEDIA_EMPTY: i64 = 808;
const ALERT_CODE_INPUT_MANUAL_REQUEST: i64 = 810;
const ALERT_GROUP_INPUT: i64 = 8;
const ALERT_SEVERITY_WARNING: i64 = 4;

#[derive(Debug, Default, Clone)]
pub struct PrinterPollDetails {
    pub toners: Vec<TonerSupply>,
    pub detected_make: Option<String>,
    pub detected_model: Option<String>,
    pub serial_number: Option<String>,
    pub page_count: Option<u64>,
    pub printer_state: Option<String>,
    pub status_message: Option<String>,
    pub paper_jam: bool,
    pub out_of_paper: bool,
    pub is_color: Option<bool>,
    pub is_duplex: Option<bool>,
}

impl PrinterPollDetails {
    fn merge_ipp_fallback(&mut self, ipp_data: PrinterPollDetails) {
        if self.toners.is_empty() {
            self.toners = ipp_data.toners;
        }
        if self.detected_make.is_none() {
            self.detected_make = ipp_data.detected_make;
        }
        if self.detected_model.is_none() {
            self.detected_model = ipp_data.detected_model;
        }
        if self.printer_state.is_none() {
            self.printer_state = ipp_data.printer_state;
        }
        if self.status_message.is_none() {
            self.status_message = ipp_data.status_message;
        }
        if self.is_color.is_none() {
            self.is_color = ipp_data.is_color;
        }
        if self.is_duplex.is_none() {
            self.is_duplex = ipp_data.is_duplex;
        }
        self.paper_jam |= ipp_data.paper_jam;
        self.out_of_paper |= ipp_data.out_of_paper;
    }
}

async fn poll_general_info(
    sock: &UdpSocket,
    community: &str,
    req_id: &mut i32,
    details: &mut PrinterPollDetails,
) -> Option<BTreeMap<String, u8>> {
    let general_oids: Vec<Vec<u32>> = [
        OID_HR_DEVICE_DESCR,
        OID_HR_DEVICE_STATUS,
        OID_PRT_GENERAL_PRINTER_NAME,
        OID_PRT_GENERAL_SERIAL_NUMBER,
        OID_PRT_MARKER_LIFE_COUNT,
        OID_PRT_MARKER_PROCESS_COLORANTS,
        OID_HR_PRINTER_STATUS,
        OID_HR_PRINTER_ERROR_STATE,
        OID_PRT_CONSOLE_LINE_1,
        OID_PRT_CONSOLE_LINE_2,
        OID_SYS_DESCR,
        OID_BROTHER_MAINTENANCE,
        OID_PRT_INPUT_VENDOR_NAME,
        OID_ENT_PHYSICAL_MFG_NAME,
    ]
    .into_iter()
    .map(|oid| oid.to_vec())
    .collect();

    let resp = snmp_request(sock, community, *req_id, PDU_GET_REQUEST, &general_oids).await?;
    *req_id += 1;

    let mut hr_device_descr = None;
    let mut prt_general_name = None;
    let mut sys_descr = None;
    let mut vendor_oid_name = None;
    let mut hr_device_status = None;
    let mut hr_printer_status = None;
    let mut hr_error_bytes: Option<Vec<u8>> = None;
    let mut console_lines: Vec<String> = Vec::new();
    let mut brother_toner_pcts = BTreeMap::<String, u8>::new();

    for (oid, val) in resp {
        let Some(v) = val else { continue };
        let oid_slice = oid.as_slice();

        if oid_slice == OID_HR_DEVICE_DESCR {
            hr_device_descr = v.as_str().filter(|s| !s.is_empty());
        } else if oid_slice == OID_HR_DEVICE_STATUS {
            hr_device_status = v.as_i64();
        } else if oid_slice == OID_PRT_GENERAL_PRINTER_NAME {
            prt_general_name = v.as_str().filter(|s| !s.is_empty());
        } else if oid_slice == OID_PRT_GENERAL_SERIAL_NUMBER {
            details.serial_number = v.as_str().filter(|s| !s.is_empty());
        } else if oid_slice == OID_PRT_MARKER_LIFE_COUNT {
            details.page_count = v.as_i64().filter(|&c| c >= 0).map(|c| c as u64);
        } else if oid_slice == OID_PRT_MARKER_PROCESS_COLORANTS {
            if let Some(c) = v.as_i64() {
                if c > 1 {
                    details.is_color = Some(true);
                } else if c == 1 {
                    details.is_color = Some(false);
                }
            }
        } else if oid_slice == OID_HR_PRINTER_STATUS {
            if let Some(st) = v.as_i64() {
                hr_printer_status = Some(st);
                let state_str = if st == HR_PRINTER_STATUS_PRINTING {
                    "Printing"
                } else {
                    "Idle"
                };
                details.printer_state = Some(state_str.to_string());
            }
        } else if oid_slice == OID_HR_PRINTER_ERROR_STATE {
            hr_error_bytes = v.as_bytes().map(|b| b.to_vec());
        } else if oid_slice == OID_PRT_CONSOLE_LINE_1 || oid_slice == OID_PRT_CONSOLE_LINE_2 {
            if let Some(s) = v.as_str().filter(|s| !s.is_empty())
                && !console_lines.contains(&s)
            {
                console_lines.push(s);
            }
        } else if oid_slice == OID_SYS_DESCR {
            sys_descr = v.as_str().filter(|s| !s.is_empty());
        } else if oid_slice == OID_BROTHER_MAINTENANCE {
            if let Some(bytes) = v.as_bytes() {
                brother_toner_pcts = parse_brother_maintenance_bytes(bytes);
            }
        } else if (oid_slice == OID_PRT_INPUT_VENDOR_NAME
            || oid_slice == OID_ENT_PHYSICAL_MFG_NAME)
            && vendor_oid_name.is_none()
        {
            vendor_oid_name = v.as_str().filter(|s| !s.is_empty());
        }
    }

    if !console_lines.is_empty() {
        let joined = console_lines.join(" — ");
        if is_paper_jam_text(&joined) {
            details.paper_jam = true;
        }
        if is_out_of_paper_text(&joined) {
            details.out_of_paper = true;
        }
        details.status_message = Some(joined);
    }

    if let Some(ref err_bytes) = hr_error_bytes {
        let (jam, no_paper) =
            parse_hr_error_state(err_bytes, hr_device_status, hr_printer_status);
        details.paper_jam |= jam;
        details.out_of_paper |= no_paper;
    }

    details.detected_make = hr_device_descr
        .as_deref()
        .and_then(extract_printer_make)
        .or_else(|| sys_descr.as_deref().and_then(extract_printer_make))
        .or_else(|| vendor_oid_name.as_deref().and_then(clean_vendor_name));

    details.detected_model = hr_device_descr
        .or_else(|| {
            sys_descr.map(|s| {
                s.split([';', ',', '\n'])
                    .next()
                    .unwrap_or(&s)
                    .trim()
                    .to_string()
            })
        })
        .or(prt_general_name);

    Some(brother_toner_pcts)
}

async fn poll_colorants(
    sock: &UdpSocket,
    community: &str,
    req_id: &mut i32,
) -> BTreeMap<u32, String> {
    let mut colorant_oid = vec![OID_PRT_MARKER_COLORANT_VALUE.to_vec()];
    let mut colorants = BTreeMap::new();

    for _ in 0..MAX_COLORANT_WALK_STEPS {
        let Some(c_resp) =
            snmp_request(sock, community, *req_id, PDU_GET_NEXT_REQUEST, &colorant_oid).await
        else {
            break;
        };
        *req_id += 1;

        let Some((ret_oid, Some(val))) = c_resp.into_iter().next() else {
            break;
        };
        if ret_oid.len() > OID_PRT_MARKER_COLORANT_VALUE.len()
            && ret_oid.starts_with(OID_PRT_MARKER_COLORANT_VALUE)
        {
            if let Some(&last_idx) = ret_oid.last()
                && let Some(s) = val.as_str()
            {
                colorants.insert(last_idx, s);
            }
            colorant_oid = vec![ret_oid];
        } else {
            break;
        }
    }

    colorants
}

async fn poll_supplies(
    sock: &UdpSocket,
    community: &str,
    req_id: &mut i32,
    brother_toner_pcts: &BTreeMap<String, u8>,
    details: &mut PrinterPollDetails,
) {
    #[derive(Default)]
    struct RawSupply {
        colorant_idx: Option<i64>,
        supply_type: Option<i64>,
        description: Option<String>,
        max_capacity: Option<i32>,
        level: Option<i32>,
    }

    let mut supplies_map: BTreeMap<Vec<u32>, RawSupply> = BTreeMap::new();
    snmp_walk_columns(
        sock,
        community,
        req_id,
        OID_PRT_MARKER_SUPPLIES_ENTRY,
        SUPPLY_WALK_COLUMNS,
        MAX_SUPPLY_WALK_STEPS,
        |instance_key, col, val| {
            let entry = supplies_map.entry(instance_key).or_default();
            match col {
                SUPPLY_COL_COLORANT_INDEX => entry.colorant_idx = val.as_i64(),
                SUPPLY_COL_TYPE => entry.supply_type = val.as_i64(),
                SUPPLY_COL_DESCRIPTION => entry.description = val.as_str(),
                SUPPLY_COL_MAX_CAPACITY => entry.max_capacity = val.as_i64().map(|v| v as i32),
                SUPPLY_COL_LEVEL => entry.level = val.as_i64().map(|v| v as i32),
                _ => {}
            }
        },
    )
    .await;

    let colorants = poll_colorants(sock, community, req_id).await;

    for (_instance, raw) in supplies_map {
        let mut level = raw.level.unwrap_or(SUPPLY_LEVEL_UNKNOWN);
        let mut max_capacity = raw.max_capacity.unwrap_or(DEFAULT_MAX_CAPACITY);
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
                            Some(f) => {
                                format!("{} Toner", f.to_uppercase().collect::<String>() + ch.as_str())
                            }
                            None => "Toner".to_string(),
                        }
                    })
                    .unwrap_or_else(|| "Toner Supply".to_string())
            });

        let supply_type = supply_type_label(raw.supply_type, &desc);
        if !is_primary_toner_or_ink(&supply_type, &desc) {
            continue;
        }

        let color = infer_supply_color(&desc, colorant_str);
        let mut percent = compute_percent(level, max_capacity);

        if percent.is_none()
            && let Some(&br_pct) = brother_toner_pcts.get(&color)
        {
            percent = Some(br_pct);
            level = i32::from(br_pct);
            max_capacity = DEFAULT_MAX_CAPACITY;
        }

        details.toners.push(TonerSupply {
            name: desc,
            color,
            level,
            max_capacity,
            percent,
            supply_type,
        });
    }

    if details.toners.is_empty() && !brother_toner_pcts.is_empty() {
        for (color, label) in [
            ("black", "Black Toner Cartridge"),
            ("cyan", "Cyan Toner Cartridge"),
            ("magenta", "Magenta Toner Cartridge"),
            ("yellow", "Yellow Toner Cartridge"),
        ] {
            if let Some(&pct) = brother_toner_pcts.get(color) {
                details.toners.push(TonerSupply {
                    name: label.to_string(),
                    color: color.to_string(),
                    level: i32::from(pct),
                    max_capacity: DEFAULT_MAX_CAPACITY,
                    percent: Some(pct),
                    supply_type: "toner".to_string(),
                });
            }
        }
    }

    if details.is_color.is_none() {
        let has_cmy_toner = details.toners.iter().any(|t| {
            matches!(t.color.as_str(), "cyan" | "magenta" | "yellow")
        });
        if has_cmy_toner {
            details.is_color = Some(true);
        } else if !colorants.is_empty() {
            let has_cmy_colorant = colorants.values().any(|val| {
                let v = val.to_lowercase();
                v.contains("cyan") || v.contains("magenta") || v.contains("yellow")
            });
            if has_cmy_colorant {
                details.is_color = Some(true);
            } else if colorants.values().any(|v| v.to_lowercase().contains("black")) {
                details.is_color = Some(false);
            }
        } else if !details.toners.is_empty() {
            let only_black = details.toners.iter().all(|t| {
                matches!(t.color.as_str(), "black" | "waste" | "drum" | "maintenance")
            });
            if only_black {
                details.is_color = Some(false);
            }
        }
    }
}

async fn poll_alerts(
    sock: &UdpSocket,
    community: &str,
    req_id: &mut i32,
    details: &mut PrinterPollDetails,
) {
    #[derive(Default)]
    struct RawAlert {
        severity: Option<i64>,
        group: Option<i64>,
        code: Option<i64>,
        description: Option<String>,
    }

    let mut alerts_map: BTreeMap<Vec<u32>, RawAlert> = BTreeMap::new();
    snmp_walk_columns(
        sock,
        community,
        req_id,
        OID_PRT_ALERT_ENTRY,
        ALERT_WALK_COLUMNS,
        MAX_ALERT_WALK_STEPS,
        |instance_key, col, val| {
            let entry = alerts_map.entry(instance_key).or_default();
            match col {
                ALERT_COL_SEVERITY => entry.severity = val.as_i64(),
                ALERT_COL_GROUP => entry.group = val.as_i64(),
                ALERT_COL_CODE => entry.code = val.as_i64(),
                ALERT_COL_DESCRIPTION => entry.description = val.as_str().filter(|s| !s.is_empty()),
                _ => {}
            }
        },
    )
    .await;

    for (_instance, alert) in alerts_map {
        if alert.code == Some(ALERT_CODE_JAM) {
            details.paper_jam = true;
        }
        let is_input_empty_alert = matches!(
            alert.code,
            Some(ALERT_CODE_INPUT_MEDIA_EMPTY | ALERT_CODE_INPUT_MANUAL_REQUEST)
        ) || (alert.group == Some(ALERT_GROUP_INPUT)
            && alert.code == Some(ALERT_CODE_SUBUNIT_EMPTY)
            && alert.severity != Some(ALERT_SEVERITY_WARNING));

        if is_input_empty_alert {
            details.out_of_paper = true;
        }
        if let Some(ref desc) = alert.description {
            let mut matched = false;
            if is_paper_jam_text(desc) {
                details.paper_jam = true;
                matched = true;
            }
            if is_out_of_paper_text(desc) {
                details.out_of_paper = true;
                matched = true;
            }
            if matched && details.status_message.is_none() {
                details.status_message = Some(desc.clone());
            }
        }
    }
}

async fn poll_media_paths(
    sock: &UdpSocket,
    community: &str,
    req_id: &mut i32,
    details: &mut PrinterPollDetails,
) {
    #[derive(Default)]
    struct RawMediaPath {
        path_type: Option<i64>,
        description: Option<String>,
    }

    let mut paths_map: BTreeMap<Vec<u32>, RawMediaPath> = BTreeMap::new();
    snmp_walk_columns(
        sock,
        community,
        req_id,
        OID_PRT_MEDIA_PATH_ENTRY,
        MEDIA_PATH_WALK_COLUMNS,
        MAX_MEDIA_PATH_WALK_STEPS,
        |instance_key, col, val| {
            let entry = paths_map.entry(instance_key).or_default();
            match col {
                MEDIA_PATH_COL_TYPE => entry.path_type = val.as_i64(),
                MEDIA_PATH_COL_DESCRIPTION => entry.description = val.as_str(),
                _ => {}
            }
        },
    )
    .await;

    let mut has_duplex = false;
    let mut has_simplex = false;

    for (_instance, raw) in paths_map {
        let is_duplex_type = matches!(raw.path_type, Some(3 | 4));
        let is_duplex_desc = raw
            .description
            .as_deref()
            .map(|d| d.to_lowercase().contains("duplex"))
            .unwrap_or(false);

        if is_duplex_type || is_duplex_desc {
            has_duplex = true;
        }

        let is_simplex_type = raw.path_type == Some(5);
        let is_simplex_desc = raw
            .description
            .as_deref()
            .map(|d| d.to_lowercase().contains("simplex"))
            .unwrap_or(false);

        if is_simplex_type || is_simplex_desc {
            has_simplex = true;
        }
    }

    if has_duplex {
        details.is_duplex = Some(true);
    } else if has_simplex {
        details.is_duplex = Some(false);
    }
}

fn apply_capability_fallbacks(details: &mut PrinterPollDetails) {
    if details.is_color.is_none() {
        let has_cmy_toner = details.toners.iter().any(|t| {
            matches!(t.color.as_str(), "cyan" | "magenta" | "yellow")
        });
        if has_cmy_toner {
            details.is_color = Some(true);
        } else if !details.toners.is_empty() {
            let only_black = details.toners.iter().all(|t| {
                matches!(t.color.as_str(), "black" | "waste" | "drum" | "maintenance")
            });
            if only_black {
                details.is_color = Some(false);
            }
        }
    }

    let (inferred_color, inferred_duplex) = infer_capabilities_from_model(
        details.detected_model.as_deref(),
        details.status_message.as_deref(),
    );

    if details.is_color.is_none() {
        details.is_color = inferred_color;
    }
    if details.is_duplex.is_none() {
        details.is_duplex = inferred_duplex;
    }
}

async fn poll_snmp(host: &str, community: &str) -> Option<PrinterPollDetails> {
    let host_owned = host.to_string();
    let target_addr = tokio::task::spawn_blocking(move || {
        if let Ok(ip) = host_owned.parse::<std::net::IpAddr>() {
            Some(std::net::SocketAddr::new(ip, SNMP_PORT))
        } else {
            format!("{host_owned}:{SNMP_PORT}")
                .to_socket_addrs()
                .ok()?
                .next()
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
    let mut req_id = INITIAL_SNMP_REQUEST_ID;

    let brother_toner_pcts = poll_general_info(&sock, community, &mut req_id, &mut details).await?;
    poll_supplies(
        &sock,
        community,
        &mut req_id,
        &brother_toner_pcts,
        &mut details,
    )
    .await;
    poll_media_paths(&sock, community, &mut req_id, &mut details).await;
    poll_alerts(&sock, community, &mut req_id, &mut details).await;

    Some(details)
}

pub async fn poll_printer_details(host: &str, community: &str) -> PrinterPollDetails {
    let comm = normalize_snmp_community(community);

    let mut details = if let Some(mut snmp_data) = poll_snmp(host, comm).await {
        if (snmp_data.toners.is_empty()
            || snmp_data.is_color.is_none()
            || snmp_data.is_duplex.is_none())
            && let Some(ipp_data) = poll_ipp(host).await
        {
            snmp_data.merge_ipp_fallback(ipp_data);
        }
        snmp_data
    } else {
        poll_ipp(host).await.unwrap_or_default()
    };

    apply_capability_fallbacks(&mut details);

    details
}

