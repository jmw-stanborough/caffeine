pub mod auth;
pub mod routes;
pub mod types;

use routes::{create_router, ServerAppState};
use rusqlite::Connection;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
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
        app: Option<AppHandle>,
        bind_ip: &str,
        port: u16,
        db_path: PathBuf,
    ) -> Result<SocketAddr, String> {
        let instance = start_embedded_server(app, bind_ip, port, db_path, None).await?;
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
    app: Option<AppHandle>,
    bind_ip: &str,
    port: u16,
    db_path: PathBuf,
    admin_token_override: Option<String>,
) -> Result<ServerInstance, String> {
    let conn = init_server_db(&db_path)?;
    crate::db::init_db_schema(&conn)?;

    let (broadcast_tx, _) = broadcast::channel::<ServerBroadcastEvent>(512);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let admin_token = admin_token_override.or_else(|| {
        crate::config::load_machine_settings()
            .ok()
            .and_then(|m| m.admin_token)
            .filter(|t| !t.trim().is_empty())
    });

    let app_state = Arc::new(ServerAppState {
        db: Mutex::new(conn),
        broadcast_tx: broadcast_tx.clone(),
        app,
        nonce_cache: Mutex::new(auth::NonceCache::new()),
        admin_token,
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
            .start(None, "127.0.0.1", 0, db_path.clone())
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

    #[tokio::test]
    async fn test_hmac_auth_mutations_and_rejections() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("caffeine_test_hmac_{nanos}.db"));
        let token = "caff_integration_test_secret";

        let server_instance = start_embedded_server(
            None,
            "127.0.0.1",
            0,
            db_path.clone(),
            Some(token.to_string()),
        )
        .await
        .expect("Failed to start test server");

        let addr = server_instance.bound_addr;
        let client = reqwest::Client::new();
        let servers_url = format!("http://{addr}/api/v1/servers");
        let server_payload = r#"{"id":"srv-1","host":"10.0.0.1","label":"Test Server","description":null,"agent":null,"sort_order":0}"#;

        let bearer_res = client
            .post(&servers_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(server_payload)
            .send()
            .await
            .unwrap();
        assert_eq!(bearer_res.status(), reqwest::StatusCode::UNAUTHORIZED);

        let no_auth_res = client
            .post(&servers_url)
            .header("Content-Type", "application/json")
            .body(server_payload)
            .send()
            .await
            .unwrap();
        assert_eq!(no_auth_res.status(), reqwest::StatusCode::UNAUTHORIZED);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let nonce = "test_unique_nonce_1";
        let canonical = crate::crypto::compute_canonical_string(
            "POST",
            "/api/v1/servers",
            timestamp,
            nonce,
            server_payload.as_bytes(),
        );
        let signature = crate::crypto::compute_hmac_sha256(token, &canonical).unwrap();

        let valid_res = client
            .post(&servers_url)
            .header("X-Caffeine-Timestamp", timestamp.to_string())
            .header("X-Caffeine-Nonce", nonce)
            .header("X-Caffeine-Signature", signature.clone())
            .header("Content-Type", "application/json")
            .body(server_payload)
            .send()
            .await
            .unwrap();
        assert_eq!(valid_res.status(), reqwest::StatusCode::OK);

        let replay_res = client
            .post(&servers_url)
            .header("X-Caffeine-Timestamp", timestamp.to_string())
            .header("X-Caffeine-Nonce", nonce)
            .header("X-Caffeine-Signature", signature)
            .header("Content-Type", "application/json")
            .body(server_payload)
            .send()
            .await
            .unwrap();
        assert_eq!(replay_res.status(), reqwest::StatusCode::UNAUTHORIZED);

        let mut server_instance = server_instance;
        server_instance.stop();
        let _ = std::fs::remove_file(db_path);
    }
}
