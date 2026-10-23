pub mod ber;
pub mod ipp;
pub mod oids;
pub mod parsers;
pub mod walk;

pub use parsers::{
    clean_vendor_name, extract_printer_make, infer_capabilities_from_model, infer_supply_color,
    is_out_of_paper_text, is_paper_jam_text, is_primary_toner_or_ink,
    parse_brother_maintenance_bytes, parse_hr_error_state,
};

use super::types::{TonerSupply, normalize_snmp_community};
use ipp::poll_ipp;
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use walk::{poll_alerts, poll_general_info, poll_media_paths, poll_supplies};

const SNMP_PORT: u16 = 161;
const INITIAL_SNMP_REQUEST_ID: i32 = 100;

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

fn apply_capability_fallbacks(details: &mut PrinterPollDetails) {
    if details.is_color.is_none() {
        let has_cmy_toner = details
            .toners
            .iter()
            .any(|t| matches!(t.color.as_str(), "cyan" | "magenta" | "yellow"));
        if has_cmy_toner {
            details.is_color = Some(true);
        } else if !details.toners.is_empty() {
            let only_black = details
                .toners
                .iter()
                .all(|t| matches!(t.color.as_str(), "black" | "waste" | "drum" | "maintenance"));
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

    let brother_toner_pcts =
        poll_general_info(&sock, community, &mut req_id, &mut details).await?;
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
