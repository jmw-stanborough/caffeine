use super::types::{Printer, PrinterLog};
use rusqlite::{Connection, OptionalExtension, params};

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
            Ok(Printer {
                id: row.get(0)?,
                host: row.get(1)?,
                label: row.get(2)?,
                description: row.get(3)?,
                model: row.get(4)?,
                location: row.get(5)?,
                snmp_community: community
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "public".to_string()),
                web_url: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn insert_printer(conn: &Connection, printer: &Printer) -> Result<(), String> {
    let id = if printer.id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        printer.id.clone()
    };

    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM printers",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let community = if printer.snmp_community.trim().is_empty() {
        "public"
    } else {
        printer.snmp_community.trim()
    };

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
    let community = if printer.snmp_community.trim().is_empty() {
        "public"
    } else {
        printer.snmp_community.trim()
    };

    conn.execute(
        "UPDATE printers
         SET host = ?1, label = ?2, description = ?3, model = ?4, location = ?5, snmp_community = ?6, web_url = ?7
         WHERE id = ?8",
        params![
            printer.host,
            printer.label,
            printer.description,
            printer.model,
            printer.location,
            community,
            printer.web_url,
            id
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_printer_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM printers WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_printer_sort_order(conn: &mut Connection, printers: &[Printer]) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("UPDATE printers SET sort_order = ?1 WHERE id = ?2")
            .map_err(|e| e.to_string())?;

        for (idx, printer) in printers.iter().enumerate() {
            stmt.execute(params![idx as i64, printer.id])
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_printer_log_entry(
    conn: &Connection,
    printer_id: &str,
    event_type: &str,
    latency_ms: Option<u64>,
) -> Result<String, String> {
    conn.execute(
        "INSERT INTO printer_logs (printer_id, event_type, latency_ms) VALUES (?1, ?2, ?3)",
        params![printer_id, event_type, latency_ms.map(|l| l as i64)],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT timestamp FROM printer_logs WHERE id = last_insert_rowid()",
        [],
        |r| r.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn get_latest_printer_log(
    conn: &Connection,
    printer_id: &str,
) -> Result<Option<(String, String)>, String> {
    conn.query_row(
        "SELECT event_type, timestamp FROM printer_logs
         WHERE printer_id = ?1
         ORDER BY id DESC LIMIT 1",
        params![printer_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn query_printer_logs(
    conn: &Connection,
    printer_id: &str,
    limit: i64,
) -> Result<Vec<PrinterLog>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, printer_id, event_type, latency_ms, timestamp
             FROM printer_logs
             WHERE printer_id = ?1
             ORDER BY id DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![printer_id, limit], |row| {
            let lat: Option<i64> = row.get(3)?;
            Ok(PrinterLog {
                id: row.get(0)?,
                printer_id: row.get(1)?,
                event_type: row.get(2)?,
                latency_ms: lat.map(|l| l as u64),
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
