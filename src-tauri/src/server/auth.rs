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
}
