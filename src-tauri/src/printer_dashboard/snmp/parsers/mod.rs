pub mod alerts;
pub mod supplies;
pub mod vendor;

pub use alerts::*;
pub use supplies::*;
pub use vendor::*;

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

        assert_eq!(
            infer_capabilities_from_model(Some("Brother HL-L3240CDW series"), None),
            (Some(true), Some(true))
        );
        assert_eq!(
            infer_capabilities_from_model(Some("HP LaserJet P2055dn"), Some("Office mono printer")),
            (Some(false), Some(true))
        );
        assert_eq!(
            infer_capabilities_from_model(Some("Brother HL-1110"), Some("Simplex monochrome desktop")),
            (Some(false), Some(false))
        );
        assert_eq!(
            infer_capabilities_from_model(Some("HP LaserJet 200 color M251n"), None),
            (Some(true), None)
        );
    }
}
