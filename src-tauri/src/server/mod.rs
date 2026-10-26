pub mod auth;
pub mod routes;
pub mod types;

use routes::{create_router, ServerAppState};
use rusqlite::Connection;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use types::ServerBroadcastEvent;

pub struct ServerInstance {
    pub broadcast_tx: broadcast::Sender<ServerBroadcastEvent>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ServerInstance {
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub fn init_server_db(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Failed to open server DB: {e}"))?;

    conn.execute_batch(
        "
        PRAGMA busy_timeout = 5000;
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS server_users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'admin',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS server_sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES server_users(id) ON DELETE CASCADE,
            expires_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| format!("Failed to initialize server users schema: {e}"))?;

    Ok(conn)
}

pub async fn start_embedded_server(
    bind_ip: &str,
    port: u16,
    db_path: PathBuf,
) -> Result<ServerInstance, String> {
    let conn = init_server_db(&db_path)?;
    crate::db::init_db_schema(&conn)?;

    let (broadcast_tx, _) = broadcast::channel::<ServerBroadcastEvent>(512);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let app_state = Arc::new(ServerAppState {
        db: Mutex::new(conn),
        broadcast_tx: broadcast_tx.clone(),
    });

    let router = create_router(app_state);

    let addr: SocketAddr = format!("{bind_ip}:{port}")
        .parse()
        .map_err(|e| format!("Invalid bind address '{bind_ip}:{port}': {e}"))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind TCP listener to {addr}: {e}"))?;

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    Ok(ServerInstance {
        broadcast_tx,
        shutdown_tx: Some(shutdown_tx),
    })
}
