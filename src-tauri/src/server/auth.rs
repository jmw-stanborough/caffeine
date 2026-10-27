use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;

pub const MAX_CLOCK_DRIFT_SECS: u64 = 60;
pub const NONCE_EXPIRY_SECS: u64 = 120;

#[derive(Debug, Default)]
pub struct NonceCache {
    entries: HashMap<String, u64>,
}

impl NonceCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn check_and_record(
        &mut self,
        nonce: &str,
        timestamp: u64,
        current_time: u64,
    ) -> Result<(), &'static str> {
        let drift = if current_time >= timestamp {
            current_time - timestamp
        } else {
            timestamp - current_time
        };

        if drift > MAX_CLOCK_DRIFT_SECS {
            return Err("Request timestamp expired or skewed beyond allowable window");
        }

        self.entries.retain(|_, &mut ts| {
            current_time.saturating_sub(ts) <= NONCE_EXPIRY_SECS
        });

        if self.entries.contains_key(nonce) {
            return Err("Replay attack detected: nonce already utilized");
        }

        self.entries.insert(nonce.to_string(), timestamp);
        Ok(())
    }
}

pub fn validate_hmac_signature(
    token: &str,
    method: &str,
    uri_path: &str,
    timestamp: u64,
    nonce: &str,
    body: &[u8],
    signature: &str,
) -> Result<bool, String> {
    let canonical = crate::crypto::compute_canonical_string(method, uri_path, timestamp, nonce, body);
    crate::crypto::verify_hmac_sha256(token, &canonical, signature)
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Password hashing failed: {e}"))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| format!("Invalid password hash format: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_admin_user_if_missing(
    conn: &Connection,
    username: &str,
    initial_password: &str,
) -> Result<(), String> {
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM server_users WHERE username = ?1",
            params![username],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count == 0 {
        let hash = hash_password(initial_password)?;
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO server_users (id, username, password_hash, role) VALUES (?1, ?2, ?3, 'admin')",
            params![id, username, hash],
        )
        .map_err(|e| format!("Failed to create initial admin user: {e}"))?;
    }

    Ok(())
}

pub fn authenticate_and_create_session(
    conn: &Connection,
    username: &str,
    password: &str,
) -> Result<Option<(String, String)>, String> {
    let user_row: Option<(String, String, String)> = conn
        .query_row(
            "SELECT id, password_hash, role FROM server_users WHERE username = ?1",
            params![username],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let Some((user_id, hash, role)) = user_row else {
        return Ok(None);
    };

    if !verify_password(password, &hash)? {
        return Ok(None);
    }

    let token = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO server_sessions (token, user_id, expires_at)
         VALUES (?1, ?2, datetime('now', '+30 days'))",
        params![token, user_id],
    )
    .map_err(|e| format!("Failed to store session token: {e}"))?;

    Ok(Some((token, role)))
}

pub fn validate_admin_token(conn: &Connection, token: &str) -> Result<bool, String> {
    if let Ok(machine) = crate::config::load_machine_settings() {
        if let Some(configured_token) = machine.admin_token {
            if !configured_token.trim().is_empty() && crate::crypto::constant_time_eq(token, configured_token.trim()) {
                return Ok(true);
            }
        }
    }

    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM server_sessions s
             JOIN server_users u ON s.user_id = u.id
             WHERE s.token = ?1 AND u.role = 'admin' AND s.expires_at > datetime('now')",
            params![token],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_cache_check_and_record() {
        let mut cache = NonceCache::new();
        let now = 1730000000;

        assert!(cache.check_and_record("nonce1", now, now).is_ok());
        assert!(cache.check_and_record("nonce1", now, now).is_err());
        assert!(cache.check_and_record("nonce2", now, now).is_ok());
    }

    #[test]
    fn test_nonce_cache_rejects_clock_drift() {
        let mut cache = NonceCache::new();
        let now = 1730000000;

        assert!(cache.check_and_record("nonce_old", now - 61, now).is_err());
        assert!(cache.check_and_record("nonce_future", now + 61, now).is_err());
        assert!(cache.check_and_record("nonce_valid", now - 59, now).is_ok());
    }

    #[test]
    fn test_nonce_cache_prunes_expired() {
        let mut cache = NonceCache::new();
        let t1 = 1730000000;
        let t2 = t1 + NONCE_EXPIRY_SECS + 5;

        assert!(cache.check_and_record("nonce_expiring", t1, t1).is_ok());
        assert!(cache.check_and_record("nonce_fresh", t2, t2).is_ok());
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.entries.contains_key("nonce_fresh"));
        assert!(!cache.entries.contains_key("nonce_expiring"));
    }

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("hashing should succeed");
        assert!(verify_password(password, &hash).expect("verification should run"));
        assert!(!verify_password("WrongPassword", &hash).expect("wrong password should not match"));
    }
}
