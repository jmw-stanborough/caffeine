use super::types::{
    DEFAULT_SNMP_COMMUNITY, Printer, PrinterFilterConfig, PrinterLog, normalize_snmp_community,
};
use crate::db::{
    delete_by_id, ensure_entity_id, get_latest_log_entry, insert_log_entry, next_sort_order,
    query_log_entries, update_table_sort_order,
};
use rusqlite::{Connection, params};

pub const PRINTER_LOGS_TABLE: &str = "printer_logs";
pub const PRINTER_ID_COLUMN: &str = "printer_id";
const PRINTERS_TABLE: &str = "printers";
const FILTER_MODE_ALL: &str = "all";
const FILTER_MODE_ANY: &str = "any";

pub fn get_printers(conn: &Connection) -> Result<Vec<Printer>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, host, label, description, model, location, snmp_community, web_url
             FROM printers ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let community: Option<String> = row.get(6)?;
            let snmp_community = community
                .as_deref()
                .map(crate::server::types::sanitize_secret_update)
                .map(normalize_snmp_community)
                .unwrap_or(DEFAULT_SNMP_COMMUNITY)
                .to_string();

            Ok(Printer {
                id: row.get(0)?,
                host: row.get(1)?,
                label: row.get(2)?,
                description: row.get(3)?,
                model: row.get(4)?,
                location: row.get(5)?,
                snmp_community,
                web_url: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn insert_printer(conn: &Connection, printer: &Printer) -> Result<(), String> {
    let id = ensure_entity_id(&printer.id);
    let next_order = next_sort_order(conn, PRINTERS_TABLE)?;
    let community = normalize_snmp_community(&printer.snmp_community);

    conn.execute(
        "INSERT INTO printers (id, host, label, description, model, location, snmp_community, web_url, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            printer.host,
            printer.label,
            printer.description,
            printer.model,
            printer.location,
            community,
            printer.web_url,
            next_order
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn update_printer_entry(conn: &Connection, id: &str, printer: &Printer) -> Result<(), String> {
    let community = crate::server::types::sanitize_secret_update(&printer.snmp_community);

    conn.execute(
        "UPDATE printers
         SET host = ?1, label = ?2, description = ?3,
             snmp_community = COALESCE(NULLIF(?4, ''), snmp_community)
         WHERE id = ?5",
        params![
            printer.host,
            printer.label,
            printer.description,
            community,
            id
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_printer_entry(conn: &Connection, id: &str) -> Result<(), String> {
    delete_by_id(conn, PRINTERS_TABLE, id)
}

pub fn update_printer_sort_order(conn: &mut Connection, printers: &[Printer]) -> Result<(), String> {
    update_table_sort_order(conn, PRINTERS_TABLE, printers, |p| p.id.as_str())
}

pub fn add_printer_log_entry(
    conn: &Connection,
    printer_id: &str,
    event_type: &str,
    latency_ms: Option<u64>,
) -> Result<String, String> {
    insert_log_entry(
        conn,
        PRINTER_LOGS_TABLE,
        PRINTER_ID_COLUMN,
        printer_id,
        event_type,
        latency_ms,
    )
}

pub fn get_latest_printer_log(
    conn: &Connection,
    printer_id: &str,
) -> Result<Option<(String, String)>, String> {
    get_latest_log_entry(conn, PRINTER_LOGS_TABLE, PRINTER_ID_COLUMN, printer_id)
}

pub fn query_printer_logs(
    conn: &Connection,
    printer_id: &str,
    limit: i64,
) -> Result<Vec<PrinterLog>, String> {
    query_log_entries(
        conn,
        PRINTER_LOGS_TABLE,
        PRINTER_ID_COLUMN,
        printer_id,
        limit,
        |id, printer_id, event_type, latency_ms, timestamp| PrinterLog {
            id,
            printer_id,
            event_type,
            latency_ms,
            timestamp,
        },
    )
}

fn update_printer_detected_field(
    conn: &Connection,
    id: &str,
    column: &str,
    value: &str,
) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let sql = format!(
        "UPDATE printers SET {column} = ?1 WHERE id = ?2 AND ({column} IS NULL OR {column} != ?1)"
    );
    conn.execute(&sql, params![trimmed, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_printer_detected_model(
    conn: &Connection,
    id: &str,
    detected_model: &str,
) -> Result<(), String> {
    update_printer_detected_field(conn, id, "model", detected_model)
}

pub fn update_printer_detected_location(
    conn: &Connection,
    id: &str,
    detected_location: &str,
) -> Result<(), String> {
    update_printer_detected_field(conn, id, "location", detected_location)
}

pub fn get_printer_filter_tags(conn: &Connection) -> Result<PrinterFilterConfig, String> {
    let mut stmt = conn
        .prepare("SELECT tag, mode FROM printer_filter_tags ORDER BY sort_order ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut config = PrinterFilterConfig::default();
    for row in rows {
        let (tag, mode) = row.map_err(|e| e.to_string())?;
        if mode == FILTER_MODE_ANY {
            config.any_tags.push(tag);
        } else {
            config.all_tags.push(tag);
        }
    }

    Ok(config)
}

pub fn save_printer_filter_tags_entries(
    conn: &mut Connection,
    config: &PrinterFilterConfig,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        tx.execute("DELETE FROM printer_filter_tags", [])
            .map_err(|e| e.to_string())?;

        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO printer_filter_tags (tag, mode, sort_order) VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| e.to_string())?;

        for (mode, tags) in [
            (FILTER_MODE_ALL, &config.all_tags),
            (FILTER_MODE_ANY, &config.any_tags),
        ] {
            for (idx, tag) in tags.iter().enumerate() {
                let cleaned = tag.trim();
                if !cleaned.is_empty() {
                    stmt.execute(params![cleaned, mode, idx as i64])
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            CREATE TABLE printer_filter_tags (
                tag TEXT NOT NULL,
                mode TEXT NOT NULL DEFAULT 'all',
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (tag, mode)
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

        update_printer_sort_order(&mut conn, &[p2.clone(), p1.clone()]).unwrap();
        let reordered = get_printers(&conn).unwrap();
        assert_eq!(reordered[0].id, "p2");
        assert_eq!(reordered[1].id, "p1");

        let mut p1_updated = p1.clone();
        p1_updated.label = "Updated Reception".to_string();
        update_printer_entry(&conn, "p1", &p1_updated).unwrap();
        let after_update = get_printers(&conn).unwrap();
        assert_eq!(after_update[1].label, "Updated Reception");

        add_printer_log_entry(&conn, "p1", "connected", Some(12)).unwrap();
        let latest = get_latest_printer_log(&conn, "p1").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().0, "connected");

        let logs = query_printer_logs(&conn, "p1", 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].latency_ms, Some(12));

        delete_printer_entry(&conn, "p1").unwrap();
        let remaining = get_printers(&conn).unwrap();
        assert_eq!(remaining.len(), 1);
        let logs_after = query_printer_logs(&conn, "p1", 10).unwrap();
        assert!(logs_after.is_empty());

        update_printer_detected_model(&conn, "p2", "SHARP BP-50C26").unwrap();
        let after_detect = get_printers(&conn).unwrap();
        assert_eq!(after_detect[0].model.as_deref(), Some("SHARP BP-50C26"));

        update_printer_detected_location(&conn, "p2", "Overridden Room").unwrap();
        let p2_after_loc_attempt = get_printers(&conn).unwrap();
        assert_eq!(p2_after_loc_attempt[0].location.as_deref(), Some("Overridden Room"));

        update_printer_detected_model(&conn, "p2", "SHARP BP-70C45").unwrap();
        let p2_after_model_update = get_printers(&conn).unwrap();
        assert_eq!(p2_after_model_update[0].model.as_deref(), Some("SHARP BP-70C45"));

        let mut p2_config_update = p2_after_loc_attempt[0].clone();
        p2_config_update.label = "Accounting Colour".to_string();
        p2_config_update.model = Some("Ignored Model".to_string());
        p2_config_update.location = Some("Ignored Location".to_string());
        p2_config_update.web_url = Some("http://ignored".to_string());
        update_printer_entry(&conn, "p2", &p2_config_update).unwrap();
        let p2_after_config = get_printers(&conn)
            .unwrap()
            .into_iter()
            .find(|p| p.id == "p2")
            .unwrap();
        assert_eq!(p2_after_config.label, "Accounting Colour");
        assert_eq!(p2_after_config.model.as_deref(), Some("SHARP BP-70C45"));
        assert_eq!(p2_after_config.location.as_deref(), Some("Overridden Room"));
        assert_eq!(
            p2_after_config.web_url.as_deref(),
            Some("https://192.168.1.51:8443")
        );

        let p3 = Printer {
            id: "p3".to_string(),
            host: "192.168.1.52".to_string(),
            label: "Warehouse Printer".to_string(),
            description: None,
            model: None,
            location: None,
            snmp_community: "public".to_string(),
            web_url: None,
        };
        insert_printer(&conn, &p3).unwrap();
        update_printer_detected_location(&conn, "p3", "Warehouse Dock 4").unwrap();
        let p3_after_detect = get_printers(&conn)
            .unwrap()
            .into_iter()
            .find(|p| p.id == "p3")
            .unwrap();
        assert_eq!(p3_after_detect.location.as_deref(), Some("Warehouse Dock 4"));

        let initial_filters = get_printer_filter_tags(&conn).unwrap();
        assert!(initial_filters.all_tags.is_empty());
        assert!(initial_filters.any_tags.is_empty());

        let filter_config = PrinterFilterConfig {
            all_tags: vec!["SHARP".to_string(), "Toner Low".to_string()],
            any_tags: vec!["Printing".to_string(), "Idle".to_string()],
        };
        save_printer_filter_tags_entries(&mut conn, &filter_config).unwrap();
        let loaded_filters = get_printer_filter_tags(&conn).unwrap();
        assert_eq!(loaded_filters, filter_config);

        let updated_config = PrinterFilterConfig {
            all_tags: vec!["SHARP".to_string()],
            any_tags: vec!["Idle".to_string()],
        };
        save_printer_filter_tags_entries(&mut conn, &updated_config).unwrap();
        assert_eq!(get_printer_filter_tags(&conn).unwrap(), updated_config);

        let dual_config = PrinterFilterConfig {
            all_tags: vec!["SHARP".to_string()],
            any_tags: vec!["SHARP".to_string(), "Brother".to_string()],
        };
        save_printer_filter_tags_entries(&mut conn, &dual_config).unwrap();
        assert_eq!(get_printer_filter_tags(&conn).unwrap(), dual_config);
    }

    #[test]
    fn test_printer_redacted_community_normalization_and_preservation() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
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
            ",
        )
        .unwrap();

        let p_secret = Printer {
            id: "p-sec".to_string(),
            host: "192.168.1.100".to_string(),
            label: "Secure Printer".to_string(),
            description: None,
            model: None,
            location: None,
            snmp_community: "corp_secret_comm".to_string(),
            web_url: None,
        };
        insert_printer(&conn, &p_secret).unwrap();

        let edit_empty = Printer {
            label: "Secure Printer Edited".to_string(),
            snmp_community: "   ".to_string(),
            ..p_secret.clone()
        };
        update_printer_entry(&conn, "p-sec", &edit_empty).unwrap();
        assert_eq!(get_printers(&conn).unwrap()[0].snmp_community, "corp_secret_comm");

        let edit_redacted = Printer {
            label: "Secure Printer Redacted".to_string(),
            snmp_community: crate::server::types::REDACTED_SECRET.to_string(),
            ..p_secret.clone()
        };
        update_printer_entry(&conn, "p-sec", &edit_redacted).unwrap();
        let after_redacted_edit = get_printers(&conn).unwrap();
        assert_eq!(after_redacted_edit[0].label, "Secure Printer Redacted");
        assert_eq!(after_redacted_edit[0].snmp_community, "corp_secret_comm");

        conn.execute(
            "INSERT INTO printers (id, host, label, snmp_community, sort_order)
             VALUES ('p-redacted', '192.168.1.101', 'Display Normalized', ?1, 1)",
            params![crate::server::types::REDACTED_SECRET],
        )
        .unwrap();
        let printers = get_printers(&conn).unwrap();
        let normalized = printers.into_iter().find(|p| p.id == "p-redacted").unwrap();
        assert_eq!(normalized.snmp_community, DEFAULT_SNMP_COMMUNITY);
    }
}
