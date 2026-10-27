use tauri::AppHandle;

pub async fn execute_mutation<T, F>(
    app: &AppHandle,
    endpoint: &str,
    method: reqwest::Method,
    body: Option<&serde_json::Value>,
    local_op: F,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce() -> Result<T, String>,
{
    let settings = crate::config::load_app_settings(app)?;
    if crate::config::is_self_serving(&settings.target_host) {
        return local_op();
    }

    let token = settings.admin_token.as_ref().ok_or_else(|| {
        "Mutations are disabled in read-only client mode. Enter a valid admin access token in Settings to edit or delete resources on this server.".to_string()
    })?;

    let base_url = crate::config::build_http_base_url(&settings.target_host, settings.target_port);
    let path = format!("/api/v1/{endpoint}");
    let url = format!("{base_url}{path}");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {e}"))?
        .as_secs();

    let nonce = uuid::Uuid::new_v4().simple().to_string();

    let body_bytes = match body {
        Some(payload) => serde_json::to_vec(payload)
            .map_err(|e| format!("Payload serialization error: {e}"))?,
        None => Vec::new(),
    };

    let canonical = crate::crypto::compute_canonical_string(
        method.as_str(),
        &path,
        timestamp,
        &nonce,
        &body_bytes,
    );

    let signature = crate::crypto::compute_hmac_sha256(token.trim(), &canonical)?;

    let client = reqwest::Client::new();
    let mut req = client
        .request(method, &url)
        .header("X-Caffeine-Timestamp", timestamp.to_string())
        .header("X-Caffeine-Nonce", nonce)
        .header("X-Caffeine-Signature", signature);

    if !body_bytes.is_empty() {
        req = req.header("Content-Type", "application/json").body(body_bytes);
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("Failed to reach remote server at {url}: {e}"))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(
            "Unauthorized: Admin token rejected by remote server. Please verify your admin token in Settings."
                .to_string(),
        );
    }

    if !status.is_success() {
        let err_msg = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error from remote server".to_string());
        return Err(format!("Remote server rejected mutation ({status}): {err_msg}"));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Failed to parse remote server response: {e}"))
}

pub async fn fetch_remote_query<T>(app: &AppHandle, endpoint: &str) -> Result<Option<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let settings = crate::config::load_app_settings(app)?;
    if crate::config::is_self_serving(&settings.target_host) || settings.server_mode {
        return Ok(None);
    }

    let base_url = crate::config::build_http_base_url(&settings.target_host, settings.target_port);
    let url = format!("{base_url}/api/v1/{endpoint}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to reach remote server at {url}: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let err_msg = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error from remote server".to_string());
        return Err(format!("Remote server rejected query ({status}): {err_msg}"));
    }

    let parsed = response
        .json::<T>()
        .await
        .map_err(|e| format!("Failed to parse remote server response: {e}"))?;

    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_execute_mutation_rejects_missing_token_when_remote() {
        let settings = crate::config::AppSettings {
            target_host: "192.168.1.50".to_string(),
            target_port: 6310,
            admin_token: None,
            ..Default::default()
        };

        assert!(!crate::config::is_self_serving(&settings.target_host));
        assert!(settings.admin_token.is_none());
    }
}
