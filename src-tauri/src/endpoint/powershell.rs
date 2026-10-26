use serde::de::DeserializeOwned;
use std::process::Command;

pub fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to spawn PowerShell: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stderr.is_empty() {
            return Err(stderr);
        }
        return Err(format!(
            "PowerShell command exited with status {:?}",
            output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn extract_json_slice(stdout: &str) -> Result<&str, String> {
    let object_bounds = match (stdout.find('{'), stdout.rfind('}')) {
        (Some(start), Some(end)) if start <= end => Some((start, end)),
        _ => None,
    };
    let array_bounds = match (stdout.find('['), stdout.rfind(']')) {
        (Some(start), Some(end)) if start <= end => Some((start, end)),
        _ => None,
    };

    let bounds = match (object_bounds, array_bounds) {
        (Some(obj), Some(arr)) => {
            if obj.0 <= arr.0 {
                Some(obj)
            } else {
                Some(arr)
            }
        }
        (Some(obj), None) => Some(obj),
        (None, Some(arr)) => Some(arr),
        (None, None) => None,
    };

    let (start, end) = bounds.ok_or_else(|| {
        format!("No JSON structure found in PowerShell output: {stdout}")
    })?;

    Ok(&stdout[start..=end])
}

pub fn query_powershell_json<T: DeserializeOwned>(script: &str) -> Result<T, String> {
    let stdout = run_powershell(script)?;
    let json_slice = extract_json_slice(&stdout)?;
    serde_json::from_str::<T>(json_slice)
        .map_err(|e| format!("Failed to deserialize PowerShell JSON output: {e}. Output was: {json_slice}"))
}
