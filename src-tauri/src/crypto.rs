use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn compute_canonical_string(
    method: &str,
    uri_path: &str,
    timestamp: u64,
    nonce: &str,
    body: &[u8],
) -> String {
    let body_hash = compute_sha256_hex(body);
    let upper_method = method.to_ascii_uppercase();
    format!("{upper_method}\n{uri_path}\n{timestamp}\n{nonce}\n{body_hash}")
}

pub fn compute_hmac_sha256(key: &str, message: &str) -> Result<String, String> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|e| format!("HMAC key initialization failed: {e}"))?;
    mac.update(message.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn constant_time_eq(a: &str, b: &str) -> bool {
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

pub fn verify_hmac_sha256(key: &str, message: &str, expected_signature: &str) -> Result<bool, String> {
    let computed = compute_hmac_sha256(key, message)?;
    Ok(constant_time_eq(&computed, expected_signature))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256_empty() {
        let empty_hash = compute_sha256_hex(b"");
        assert_eq!(
            empty_hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_compute_canonical_string_format() {
        let canonical = compute_canonical_string(
            "post",
            "/api/v1/servers",
            1730000000,
            "nonce123",
            b"{\"label\":\"Test\"}",
        );
        let expected_body_hash = compute_sha256_hex(b"{\"label\":\"Test\"}");
        assert_eq!(
            canonical,
            format!("POST\n/api/v1/servers\n1730000000\nnonce123\n{expected_body_hash}")
        );
    }

    #[test]
    fn test_compute_and_verify_hmac() {
        let key = "caff_secret_admin_token";
        let message = "POST\n/api/v1/printers\n1730000000\nnonce_abc\ne3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = compute_hmac_sha256(key, message).expect("HMAC calculation should succeed");

        assert!(verify_hmac_sha256(key, message, &signature).expect("Verification should succeed"));
        assert!(!verify_hmac_sha256("wrong_key", message, &signature).expect("Wrong key should not verify"));
        assert!(!verify_hmac_sha256(key, "altered_message", &signature).expect("Altered message should not verify"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "abcd"));
    }
}
