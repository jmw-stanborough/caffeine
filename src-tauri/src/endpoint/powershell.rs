use serde::de::DeserializeOwned;
use std::process::Command;

pub fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
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

pub fn run_powershell_streaming<F>(script: &str, mut on_line: F) -> Result<(), String>
where
    F: FnMut(&str),
{
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn PowerShell: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture PowerShell stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture PowerShell stderr".to_string())?;

    // Drain stderr concurrently to prevent pipe buffer deadlocks while reading stdout.
    let stderr_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut err_str = String::new();
        let mut reader = std::io::BufReader::new(stderr);
        let _ = reader.read_to_string(&mut err_str);
        err_str
    });

    use std::io::BufRead;
    let stdout_reader = std::io::BufReader::new(stdout);
    for line in stdout_reader.lines() {
        let line = line.map_err(|e| format!("Error reading PowerShell output: {e}"))?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            on_line(trimmed);
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait on PowerShell: {e}"))?;
    let stderr_str = stderr_handle
        .join()
        .map_err(|_| "Failed to join PowerShell stderr thread".to_string())?;

    if !status.success() {
        let trimmed_err = stderr_str.trim();
        if !trimmed_err.is_empty() {
            return Err(trimmed_err.to_string());
        }
        return Err(format!(
            "PowerShell command exited with status {:?}",
            status.code()
        ));
    }

    Ok(())
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
