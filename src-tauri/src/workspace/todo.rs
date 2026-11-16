use super::current_time_ms;
use super::db::WorkspaceDbState;
use super::types::{CreateTodoInput, TodoItem, UpdateTodoInput};
use crate::db::next_sort_order;
use rusqlite::{Connection, OptionalExtension, Row, params};
use tauri::State;

const TODOS_TABLE: &str = "todos";
const DAY_MS: i64 = 86_400_000;
const WEEK_MS: i64 = 604_800_000;
const TODO_SELECT_COLUMNS: &str = "id, title, completed, completed_at, due_at, reminder_at, reminder_sent, reminder_acknowledged, recurrence, created_at, sort_order";

fn map_todo_row(row: &Row<'_>) -> rusqlite::Result<TodoItem> {
    Ok(TodoItem {
        id: row.get(0)?,
        title: row.get(1)?,
        completed: row.get::<_, i64>(2)? != 0,
        completed_at: row.get(3)?,
        due_at: row.get(4)?,
        reminder_at: row.get(5)?,
        reminder_sent: row.get::<_, i64>(6)? != 0,
        reminder_acknowledged: row.get::<_, i64>(7)? != 0,
        recurrence: row.get(8)?,
        created_at: row.get(9)?,
        sort_order: row.get(10)?,
    })
}

fn validate_title(title: &str) -> Result<String, String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("Todo title cannot be empty".to_string());
    }
    Ok(trimmed.to_string())
}

fn normalize_recurrence(recurrence: Option<String>) -> Result<Option<String>, String> {
    match recurrence {
        None => Ok(None),
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "" => Ok(None),
                "daily" | "weekly" | "monthly" => Ok(Some(normalized)),
                other => Err(format!("Unsupported todo recurrence '{other}'")),
            }
        }
    }
}

fn get_todo_by_id(conn: &Connection, id: &str) -> Result<TodoItem, String> {
    let sql = format!("SELECT {TODO_SELECT_COLUMNS} FROM {TODOS_TABLE} WHERE id = ?1");
    conn.query_row(&sql, params![id], map_todo_row)
        .optional()
        .map_err(|e| format!("Failed to load todo '{id}': {e}"))?
        .ok_or_else(|| format!("Todo '{id}' not found"))
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!("month is always in 1..=12"),
    }
}

use crate::datetime::{civil_from_days, days_from_civil};

fn add_calendar_month_ms(ts_ms: i64) -> i64 {
    let days = ts_ms.div_euclid(DAY_MS);
    let ms_of_day = ts_ms.rem_euclid(DAY_MS);
    let (mut year, mut month, day) = civil_from_days(days);
    month += 1;
    if month > 12 {
        month = 1;
        year += 1;
    }
    let clamped_day = day.min(days_in_month(year, month));
    days_from_civil(year, month, clamped_day) * DAY_MS + ms_of_day
}

fn step_recurrence_ms(ts_ms: i64, recurrence: &str) -> Result<i64, String> {
    match recurrence {
        "daily" => Ok(ts_ms + DAY_MS),
        "weekly" => Ok(ts_ms + WEEK_MS),
        "monthly" => Ok(add_calendar_month_ms(ts_ms)),
        other => Err(format!("Unsupported todo recurrence '{other}'")),
    }
}

fn advance_past_now(ts_ms: i64, recurrence: &str, now_ms: i64) -> Result<i64, String> {
    let mut next = ts_ms;
    loop {
        next = step_recurrence_ms(next, recurrence)?;
        if next > now_ms {
            return Ok(next);
        }
    }
}

pub fn load_todos_from_conn(conn: &Connection) -> Result<Vec<TodoItem>, String> {
    let sql = format!(
        "SELECT {TODO_SELECT_COLUMNS} FROM {TODOS_TABLE}
         ORDER BY completed ASC, sort_order ASC, created_at ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare todo list query: {e}"))?;

    let rows = stmt
        .query_map([], map_todo_row)
        .map_err(|e| format!("Failed to query todos: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read todo rows: {e}"))
}

pub fn create_todo_in_conn(
    conn: &Connection,
    input: CreateTodoInput,
) -> Result<TodoItem, String> {
    create_todo_at(conn, input, current_time_ms()?)
}

pub fn create_todo_at(
    conn: &Connection,
    input: CreateTodoInput,
    now_ms: i64,
) -> Result<TodoItem, String> {
    let title = validate_title(&input.title)?;
    let recurrence = normalize_recurrence(input.recurrence)?;
    let id = uuid::Uuid::new_v4().to_string();
    let sort_order = next_sort_order(conn, TODOS_TABLE)?;

    conn.execute(
        "INSERT INTO todos (id, title, completed, completed_at, due_at, reminder_at, reminder_sent, reminder_acknowledged, recurrence, created_at, sort_order)
         VALUES (?1, ?2, 0, NULL, ?3, ?4, 0, 0, ?5, ?6, ?7)",
        params![
            id,
            title,
            input.due_at,
            input.reminder_at,
            recurrence,
            now_ms,
            sort_order
        ],
    )
    .map_err(|e| format!("Failed to insert todo: {e}"))?;

    get_todo_by_id(conn, &id)
}

pub fn update_todo_in_conn(
    conn: &Connection,
    input: UpdateTodoInput,
) -> Result<TodoItem, String> {
    update_todo_at(conn, input, current_time_ms()?)
}

pub fn update_todo_at(
    conn: &Connection,
    input: UpdateTodoInput,
    now_ms: i64,
) -> Result<TodoItem, String> {
    let title = validate_title(&input.title)?;
    let recurrence = normalize_recurrence(input.recurrence)?;
    let existing = get_todo_by_id(conn, &input.id)?;

    let (reminder_sent, reminder_acknowledged) = match input.reminder_at {
        None => (false, false),
        Some(ts) if input.reminder_at != existing.reminder_at && ts > now_ms => (false, false),
        Some(_) => (existing.reminder_sent, existing.reminder_acknowledged),
    };

    conn.execute(
        "UPDATE todos
         SET title = ?1, due_at = ?2, reminder_at = ?3, reminder_sent = ?4, reminder_acknowledged = ?5, recurrence = ?6
         WHERE id = ?7",
        params![
            title,
            input.due_at,
            input.reminder_at,
            i64::from(reminder_sent),
            i64::from(reminder_acknowledged),
            recurrence,
            input.id
        ],
    )
    .map_err(|e| format!("Failed to update todo '{}': {e}", input.id))?;

    get_todo_by_id(conn, &input.id)
}

pub fn toggle_todo_in_conn(conn: &mut Connection, id: &str) -> Result<Vec<TodoItem>, String> {
    toggle_todo_at(conn, id, current_time_ms()?)
}

pub fn toggle_todo_at(
    conn: &mut Connection,
    id: &str,
    now_ms: i64,
) -> Result<Vec<TodoItem>, String> {
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start todo toggle transaction: {e}"))?;

    let existing = get_todo_by_id(&tx, id)?;
    if existing.completed {
        tx.execute(
            "UPDATE todos SET completed = 0, completed_at = NULL WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Failed to mark todo '{id}' uncompleted: {e}"))?;
    } else {
        tx.execute(
            "UPDATE todos SET completed = 1, completed_at = ?1, reminder_acknowledged = 1 WHERE id = ?2",
            params![now_ms, id],
        )
        .map_err(|e| format!("Failed to mark todo '{id}' completed: {e}"))?;

        if let Some(ref recurrence) = existing.recurrence {
            let next_due = existing
                .due_at
                .map(|due| advance_past_now(due, recurrence, now_ms))
                .transpose()?;
            let next_reminder = existing
                .reminder_at
                .map(|rem| advance_past_now(rem, recurrence, now_ms))
                .transpose()?;
            let next_id = uuid::Uuid::new_v4().to_string();
            let sort_order = next_sort_order(&tx, TODOS_TABLE)?;

            tx.execute(
                "INSERT INTO todos (id, title, completed, completed_at, due_at, reminder_at, reminder_sent, reminder_acknowledged, recurrence, created_at, sort_order)
                 VALUES (?1, ?2, 0, NULL, ?3, ?4, 0, 0, ?5, ?6, ?7)",
                params![
                    next_id,
                    existing.title,
                    next_due,
                    next_reminder,
                    recurrence,
                    now_ms,
                    sort_order
                ],
            )
            .map_err(|e| format!("Failed to insert recurring todo for '{id}': {e}"))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit todo toggle transaction: {e}"))?;

    load_todos_from_conn(conn)
}

pub fn delete_todo_in_conn(conn: &Connection, id: &str) -> Result<(), String> {
    let affected = conn
        .execute("DELETE FROM todos WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete todo '{id}': {e}"))?;
    if affected == 0 {
        return Err(format!("Todo '{id}' not found"));
    }
    Ok(())
}

pub fn list_due_reminders(conn: &Connection, now_ms: i64) -> Result<Vec<TodoItem>, String> {
    let sql = format!(
        "SELECT {TODO_SELECT_COLUMNS} FROM {TODOS_TABLE}
         WHERE completed = 0
           AND reminder_sent = 0
           AND reminder_at IS NOT NULL
           AND reminder_at <= ?1
         ORDER BY reminder_at ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare due reminders query: {e}"))?;

    let rows = stmt
        .query_map(params![now_ms], map_todo_row)
        .map_err(|e| format!("Failed to query due reminders: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read due reminder rows: {e}"))
}

pub fn mark_reminder_sent(conn: &Connection, id: &str) -> Result<(), String> {
    let affected = conn
        .execute(
            "UPDATE todos SET reminder_sent = 1, reminder_acknowledged = 0 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Failed to mark reminder sent for todo '{id}': {e}"))?;
    if affected == 0 {
        return Err(format!("Todo '{id}' not found when marking reminder sent"));
    }
    Ok(())
}

pub fn acknowledge_todo_reminder_in_conn(conn: &Connection, id: &str) -> Result<TodoItem, String> {
    let affected = conn
        .execute(
            "UPDATE todos SET reminder_acknowledged = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Failed to acknowledge reminder for todo '{id}': {e}"))?;
    if affected == 0 {
        return Err(format!("Todo '{id}' not found"));
    }
    get_todo_by_id(conn, id)
}

pub fn snooze_todo_reminder_in_conn(
    conn: &Connection,
    id: &str,
    duration_minutes: i64,
) -> Result<TodoItem, String> {
    snooze_todo_reminder_at(conn, id, duration_minutes, current_time_ms()?)
}

pub fn snooze_todo_reminder_at(
    conn: &Connection,
    id: &str,
    duration_minutes: i64,
    now_ms: i64,
) -> Result<TodoItem, String> {
    if duration_minutes <= 0 {
        return Err("Snooze duration must be greater than zero minutes".to_string());
    }
    let next_reminder = duration_minutes
        .checked_mul(60_000)
        .and_then(|offset| now_ms.checked_add(offset))
        .ok_or_else(|| "Snooze timestamp overflowed i64".to_string())?;

    let affected = conn
        .execute(
            "UPDATE todos
             SET reminder_at = ?1, reminder_sent = 0, reminder_acknowledged = 0
             WHERE id = ?2",
            params![next_reminder, id],
        )
        .map_err(|e| format!("Failed to snooze reminder for todo '{id}': {e}"))?;
    if affected == 0 {
        return Err(format!("Todo '{id}' not found"));
    }
    get_todo_by_id(conn, id)
}

#[tauri::command]
pub fn workspace_load_todos(state: State<'_, WorkspaceDbState>) -> Result<Vec<TodoItem>, String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    load_todos_from_conn(&conn)
}

#[tauri::command]
pub fn workspace_create_todo(
    state: State<'_, WorkspaceDbState>,
    input: CreateTodoInput,
) -> Result<TodoItem, String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    create_todo_in_conn(&conn, input)
}

#[tauri::command]
pub fn workspace_update_todo(
    state: State<'_, WorkspaceDbState>,
    input: UpdateTodoInput,
) -> Result<TodoItem, String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    update_todo_in_conn(&conn, input)
}

#[tauri::command]
pub fn workspace_toggle_todo(
    state: State<'_, WorkspaceDbState>,
    id: String,
) -> Result<Vec<TodoItem>, String> {
    let mut conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    toggle_todo_in_conn(&mut conn, &id)
}

#[tauri::command]
pub fn workspace_delete_todo(
    state: State<'_, WorkspaceDbState>,
    id: String,
) -> Result<(), String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    delete_todo_in_conn(&conn, &id)
}

#[tauri::command]
pub fn workspace_acknowledge_todo_reminder(
    state: State<'_, WorkspaceDbState>,
    id: String,
) -> Result<TodoItem, String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    acknowledge_todo_reminder_in_conn(&conn, &id)
}

#[tauri::command]
pub fn workspace_snooze_todo_reminder(
    state: State<'_, WorkspaceDbState>,
    id: String,
    duration_minutes: i64,
) -> Result<TodoItem, String> {
    let conn = state
        .0
        .lock()
        .map_err(|e| format!("Workspace database lock poisoned: {e}"))?;
    snooze_todo_reminder_in_conn(&conn, &id, duration_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::db::init_workspace_schema;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_workspace_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_todo_crud_and_validation() {
        let conn = memory_db();

        let empty = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "   ".to_string(),
                due_at: None,
                reminder_at: None,
                recurrence: None,
            },
            1000,
        );
        assert!(empty.is_err());

        let invalid_rec = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Task".to_string(),
                due_at: None,
                reminder_at: None,
                recurrence: Some("hourly".to_string()),
            },
            1000,
        );
        assert!(invalid_rec.is_err());

        let created = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "  Patch switches  ".to_string(),
                due_at: Some(5000),
                reminder_at: Some(4000),
                recurrence: Some("Weekly".to_string()),
            },
            1000,
        )
        .unwrap();

        assert_eq!(created.title, "Patch switches");
        assert!(!created.completed);
        assert_eq!(created.due_at, Some(5000));
        assert_eq!(created.reminder_at, Some(4000));
        assert!(!created.reminder_sent);
        assert_eq!(created.recurrence.as_deref(), Some("weekly"));

        let list = load_todos_from_conn(&conn).unwrap();
        assert_eq!(list.len(), 1);

        delete_todo_in_conn(&conn, &created.id).unwrap();
        assert!(load_todos_from_conn(&conn).unwrap().is_empty());
        assert!(delete_todo_in_conn(&conn, &created.id).is_err());
    }

    #[test]
    fn test_update_todo_resets_reminder_sent_when_moved_to_future() {
        let conn = memory_db();
        let created = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Check backup".to_string(),
                due_at: Some(10_000),
                reminder_at: Some(2_000),
                recurrence: None,
            },
            1_000,
        )
        .unwrap();

        mark_reminder_sent(&conn, &created.id).unwrap();
        let due_none = list_due_reminders(&conn, 5_000).unwrap();
        assert!(due_none.is_empty());

        // Title-only edit preserves reminder_sent = true so already-delivered notifications do not repeat.
        let title_updated = update_todo_at(
            &conn,
            UpdateTodoInput {
                id: created.id.clone(),
                title: "Check nightly backup".to_string(),
                due_at: Some(10_000),
                reminder_at: Some(2_000),
                recurrence: None,
            },
            5_000,
        )
        .unwrap();
        assert!(title_updated.reminder_sent);

        let rescheduled = update_todo_at(
            &conn,
            UpdateTodoInput {
                id: created.id.clone(),
                title: "Check nightly backup".to_string(),
                due_at: Some(20_000),
                reminder_at: Some(15_000),
                recurrence: None,
            },
            5_000,
        )
        .unwrap();
        assert!(!rescheduled.reminder_sent);
    }

    #[test]
    fn test_toggle_daily_and_weekly_recurrence_rolls_forward_past_now() {
        let mut conn = memory_db();
        let created = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Rotate logs".to_string(),
                due_at: Some(10_000),
                reminder_at: Some(5_000),
                recurrence: Some("daily".to_string()),
            },
            1_000,
        )
        .unwrap();

        // Toggling more than two days late must advance until strictly greater than now_ms.
        let now_ms = 10_000 + 2 * DAY_MS + 500;
        let all = toggle_todo_at(&mut conn, &created.id, now_ms).unwrap();
        assert_eq!(all.len(), 2);

        let completed_item = all.iter().find(|t| t.id == created.id).unwrap();
        assert!(completed_item.completed);
        assert_eq!(completed_item.completed_at, Some(now_ms));

        let next_item = all.iter().find(|t| t.id != created.id).unwrap();
        assert!(!next_item.completed);
        assert_eq!(next_item.due_at, Some(10_000 + 3 * DAY_MS));
        assert_eq!(next_item.reminder_at, Some(5_000 + 3 * DAY_MS));
        assert!(!next_item.reminder_sent);
        assert_eq!(next_item.recurrence.as_deref(), Some("daily"));

        // Toggling a completed task back to uncompleted restores its state cleanly.
        let uncompleted = toggle_todo_at(&mut conn, &created.id, now_ms + 100).unwrap();
        let restored = uncompleted.iter().find(|t| t.id == created.id).unwrap();
        assert!(!restored.completed);
        assert_eq!(restored.completed_at, None);
    }

    #[test]
    fn test_monthly_recurrence_calendar_month_clamping() {
        let jan_31 = days_from_civil(2026, 1, 31) * DAY_MS + 10 * 3_600_000;
        let feb_28 = add_calendar_month_ms(jan_31);
        assert_eq!(
            feb_28,
            days_from_civil(2026, 2, 28) * DAY_MS + 10 * 3_600_000
        );

        let leap_jan_31 = days_from_civil(2024, 1, 31) * DAY_MS + 10 * 3_600_000;
        let leap_feb_29 = add_calendar_month_ms(leap_jan_31);
        assert_eq!(
            leap_feb_29,
            days_from_civil(2024, 2, 29) * DAY_MS + 10 * 3_600_000
        );

        let dec_15 = days_from_civil(2026, 12, 15) * DAY_MS;
        let jan_15 = add_calendar_month_ms(dec_15);
        assert_eq!(jan_15, days_from_civil(2027, 1, 15) * DAY_MS);
    }

    #[test]
    fn test_list_due_reminders_filters_completed_and_sent() {
        let conn = memory_db();
        let due_task = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Due now".to_string(),
                due_at: Some(2_000),
                reminder_at: Some(1_500),
                recurrence: None,
            },
            1_000,
        )
        .unwrap();
        create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Future reminder".to_string(),
                due_at: Some(9_000),
                reminder_at: Some(8_000),
                recurrence: None,
            },
            1_000,
        )
        .unwrap();
        let done_task = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Already done".to_string(),
                due_at: Some(2_000),
                reminder_at: Some(1_200),
                recurrence: None,
            },
            1_000,
        )
        .unwrap();
        conn.execute(
            "UPDATE todos SET completed = 1 WHERE id = ?1",
            params![done_task.id],
        )
        .unwrap();

        let due = list_due_reminders(&conn, 2_000).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, due_task.id);

        mark_reminder_sent(&conn, &due_task.id).unwrap();
        assert!(list_due_reminders(&conn, 2_000).unwrap().is_empty());
    }

    #[test]
    fn test_acknowledge_and_snooze_reminder() {
        let mut conn = memory_db();
        let task = create_todo_at(
            &conn,
            CreateTodoInput {
                title: "Patch firewall".to_string(),
                due_at: Some(10_000),
                reminder_at: Some(1_500),
                recurrence: None,
            },
            1_000,
        )
        .unwrap();
        assert!(!task.reminder_acknowledged);

        mark_reminder_sent(&conn, &task.id).unwrap();
        let sent = get_todo_by_id(&conn, &task.id).unwrap();
        assert!(sent.reminder_sent);
        assert!(!sent.reminder_acknowledged);

        let acked = acknowledge_todo_reminder_in_conn(&conn, &task.id).unwrap();
        assert!(acked.reminder_acknowledged);

        let snoozed = snooze_todo_reminder_at(&conn, &task.id, 15, 2_000).unwrap();
        assert_eq!(snoozed.reminder_at, Some(2_000 + 15 * 60_000));
        assert!(!snoozed.reminder_sent);
        assert!(!snoozed.reminder_acknowledged);

        assert!(snooze_todo_reminder_at(&conn, &task.id, 0, 2_000).is_err());

        mark_reminder_sent(&conn, &task.id).unwrap();
        let items = toggle_todo_at(&mut conn, &task.id, 3_000).unwrap();
        let completed = items.iter().find(|t| t.id == task.id).unwrap();
        assert!(completed.completed);
        assert!(completed.reminder_acknowledged);
    }
}
