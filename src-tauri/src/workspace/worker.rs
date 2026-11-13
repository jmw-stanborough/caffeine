use super::current_time_ms;
use super::db::WorkspaceDbState;
use super::todo::{list_due_reminders, mark_reminder_sent};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const REMINDER_POLL_INTERVAL: Duration = Duration::from_secs(10);
const INITIAL_WORKER_DELAY: Duration = Duration::from_secs(2);
const TODOS_UPDATED_EVENT: &str = "workspace:todos-updated";

pub fn start_reminder_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(INITIAL_WORKER_DELAY).await;

        loop {
            let now_ms = match current_time_ms() {
                Ok(ts) => ts,
                Err(err) => {
                    eprintln!("Reminder worker failed to read system time: {err}");
                    tokio::time::sleep(REMINDER_POLL_INTERVAL).await;
                    continue;
                }
            };

            let due_todos = {
                let db = app.state::<WorkspaceDbState>();
                let result = {
                    let conn = match db.0.lock() {
                        Ok(guard) => guard,
                        Err(err) => panic!("Workspace database lock poisoned in reminder worker: {err}"),
                    };
                    list_due_reminders(&conn, now_ms)
                };
                match result {
                    Ok(items) => items,
                    Err(err) => {
                        eprintln!("Failed to query due todo reminders: {err}");
                        tokio::time::sleep(REMINDER_POLL_INTERVAL).await;
                        continue;
                    }
                }
            };

            let mut any_marked = false;
            for todo in due_todos {
                if let Err(err) = app
                    .notification()
                    .builder()
                    .title("Caffeine Task Reminder")
                    .body(&todo.title)
                    .show()
                {
                    eprintln!(
                        "Failed to display OS notification for todo '{}': {err}",
                        todo.id
                    );
                }

                let db = app.state::<WorkspaceDbState>();
                let mark_res = {
                    let conn = match db.0.lock() {
                        Ok(guard) => guard,
                        Err(err) => panic!("Workspace database lock poisoned in reminder worker: {err}"),
                    };
                    mark_reminder_sent(&conn, &todo.id)
                };

                match mark_res {
                    Ok(()) => any_marked = true,
                    Err(err) => eprintln!("Failed to mark reminder sent for '{}': {err}", todo.id),
                }
            }

            if any_marked
                && let Err(err) = app.emit(TODOS_UPDATED_EVENT, ())
            {
                eprintln!("Failed to emit '{TODOS_UPDATED_EVENT}': {err}");
            }

            tokio::time::sleep(REMINDER_POLL_INTERVAL).await;
        }
    });
}
