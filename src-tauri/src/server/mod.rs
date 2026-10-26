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
    pub bound_addr: SocketAddr,
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

#[derive(Clone)]
pub struct ActiveServer(pub Arc<Mutex<Option<ServerInstance>>>);

impl Default for ActiveServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveServer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub async fn start(
        &self,
        bind_ip: &str,
        port: u16,
        db_path: PathBuf,
    ) -> Result<SocketAddr, String> {
        let instance = start_embedded_server(bind_ip, port, db_path).await?;
        let addr = instance.bound_addr;

        let mut guard = self
            .0
            .lock()
            .map_err(|e| format!("Server state lock poisoned: {e}"))?;

        if let Some(mut existing) = guard.take() {
            existing.stop();
        }

        *guard = Some(instance);
        Ok(addr)
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut guard = self
            .0
            .lock()
            .map_err(|e| format!("Server state lock poisoned: {e}"))?;

        if let Some(mut existing) = guard.take() {
            existing.stop();
        }
        Ok(())
    }

    pub fn broadcast(&self, event: ServerBroadcastEvent) {
        if let Ok(guard) = self.0.lock() {
            if let Some(instance) = guard.as_ref() {
                let _ = instance.broadcast_tx.send(event);
            }
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

    let bound_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to resolve local bound address: {e}"))?;

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    Ok(ServerInstance {
        bound_addr,
        broadcast_tx,
        shutdown_tx: Some(shutdown_tx),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_active_server_lifecycle_and_snapshot_endpoint() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("caffeine_test_server_{nanos}.db"));

        let active_server = ActiveServer::new();
        let addr = active_server
            .start("127.0.0.1", 0, db_path.clone())
            .await
            .expect("Failed to start server on ephemeral port");

        let client = reqwest::Client::new();
        let snapshot_url = format!("http://{addr}/api/v1/sync/snapshot");

        let response = client
            .get(&snapshot_url)
            .send()
            .await
            .expect("Failed to query snapshot from running server");

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let body = response
            .json::<types::SnapshotResponse>()
            .await
            .expect("Failed to parse snapshot response JSON");

        assert!(body.servers.is_empty());
        assert!(body.printers.is_empty());

        active_server.stop().expect("Failed to stop server");
        let _ = std::fs::remove_file(db_path);
    }
}
