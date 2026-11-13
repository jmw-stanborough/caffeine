use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn new_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn run_command_stdout(program: &str, args: &[&str]) -> Result<String, String> {
    let output = new_hidden_command(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute '{program}': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            stderr
        };
        return if detail.is_empty() {
            Err(format!("Command '{program}' failed with status {}", output.status))
        } else {
            Err(format!(
                "Command '{program}' failed with status {}: {detail}",
                output.status
            ))
        };
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    find_executable_in_dirs(name, std::env::split_paths(&path_var))
}

fn find_executable_in_dirs<I>(name: &str, dirs: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    #[cfg(windows)]
    let has_extension = std::path::Path::new(trimmed).extension().is_some();

    for dir in dirs {
        let candidate = dir.join(trimmed);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        if !has_extension {
            let exe_candidate = dir.join(format!("{trimmed}.exe"));
            if exe_candidate.is_file() {
                return Some(exe_candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_new_hidden_command_preserves_program_and_executes() {
        #[cfg(windows)]
        let (prog, args) = ("cmd", ["/C", "echo hello"].as_slice());
        #[cfg(not(windows))]
        let (prog, args) = ("sh", ["-c", "echo hello"].as_slice());

        let cmd = new_hidden_command(prog);
        assert_eq!(cmd.get_program(), prog);

        let out = run_command_stdout(prog, args).unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn test_run_command_stdout_reports_failures() {
        let missing = run_command_stdout("caffeine_nonexistent_cmd_99999", &[]);
        assert!(missing.is_err());

        #[cfg(windows)]
        let failed = run_command_stdout("cmd", &["/C", "exit 1"]);
        #[cfg(not(windows))]
        let failed = run_command_stdout("sh", &["-c", "exit 1"]);

        assert!(failed.is_err());
    }

    #[test]
    fn test_find_executable_in_path_system_shell_and_missing() {
        #[cfg(windows)]
        {
            assert!(find_executable_in_path("cmd").is_some());
            assert!(find_executable_in_path("cmd.exe").is_some());
        }
        #[cfg(not(windows))]
        {
            assert!(find_executable_in_path("sh").is_some());
        }

        assert!(find_executable_in_path("").is_none());
        assert!(find_executable_in_path("caffeine_nonexistent_binary_99999").is_none());
    }

    #[test]
    fn test_find_executable_in_dirs_resolves_files_only() {
        let unique = format!(
            "caffeine_process_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let empty_dir = root.join("empty");
        let bin_dir = root.join("bin");
        fs::create_dir_all(&empty_dir).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();

        fs::create_dir_all(empty_dir.join("mytool")).unwrap();

        #[cfg(windows)]
        let expected = bin_dir.join("mytool.exe");
        #[cfg(not(windows))]
        let expected = bin_dir.join("mytool");
        fs::write(&expected, b"").unwrap();

        let found = find_executable_in_dirs("mytool", [empty_dir, bin_dir]);
        assert_eq!(found, Some(expected));

        let _ = fs::remove_dir_all(&root);
    }
}
