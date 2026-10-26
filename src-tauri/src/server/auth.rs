use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rusqlite::{params, Connection, OptionalExtension};

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
            if !configured_token.trim().is_empty() && constant_time_eq(token, configured_token.trim()) {
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

fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("secret_admin_token", "secret_admin_token"));
        assert!(!constant_time_eq("secret_admin_token", "wrong_admin_token"));
        assert!(!constant_time_eq("short", "longer_token"));
    }

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("hashing should succeed");
        assert!(verify_password(password, &hash).expect("verification should run"));
        assert!(!verify_password("WrongPassword", &hash).expect("wrong password should not match"));
    }
}
