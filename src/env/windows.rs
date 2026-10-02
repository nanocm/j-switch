use crate::config::{Config, same_jdk_path};
use crate::env::EnvUpdater;
use crate::error::{JdkError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::{HKEY_CURRENT_USER, REG_EXPAND_SZ, REG_SZ};
use winreg::{RegKey, RegValue};

const ENVIRONMENT_KEY: &str = "Environment";

pub struct WindowsEnvUpdater;

impl WindowsEnvUpdater {
    pub fn new() -> Self { Self }

    fn link_path() -> Result<PathBuf> {
        Ok(Config::config_dir()?.join("jsh-current"))
    }

    fn ensure_switchable_location(link: &Path) -> Result<()> {
        let directory = link.parent().ok_or_else(|| JdkError::EnvError(
            "JDK junction has no parent directory".to_string()
        ))?;
        let location = directory.to_string_lossy().replace('/', "\\").to_lowercase();
        for variable in ["ProgramFiles", "ProgramFiles(x86)", "SystemRoot"] {
            if let Some(root) = std::env::var_os(variable) {
                let root = root.to_string_lossy().trim_end_matches(['\\', '/'])
                    .replace('/', "\\").to_lowercase();
                if location == root || location.starts_with(&(root + "\\")) {
                    return Err(JdkError::EnvError(format!(
                        "Install jsh in a directory writable without Administrator rights; {} is protected",
                        directory.display()
                    )));
                }
            }
        }
        tempfile::Builder::new().prefix(".jsh-write-check-")
            .tempfile_in(directory)
            .map_err(|e| JdkError::EnvError(format!(
                "JDK switch directory {} is not writable: {e}", directory.display()
            )))?;
        Ok(())
    }

    fn environment_key() -> Result<RegKey> {
        RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(ENVIRONMENT_KEY)
            .map(|(key, _)| key)
            .map_err(|e| JdkError::EnvError(format!(
                "Cannot access user environment registry key: {e}"
            )))
    }

    fn same_env_path(a: &str, b: &str) -> bool {
        fn normalize(value: &str) -> String {
            value.trim().trim_matches('"').trim_end_matches(['\\', '/'])
                .replace('/', "\\").to_lowercase()
        }
        normalize(a) == normalize(b)
    }

    fn registry_ready(key: &RegKey, link: &Path) -> Result<bool> {
        let home: String = key.get_value("JAVA_HOME").unwrap_or_default();
        let path: String = key.get_value("Path").unwrap_or_default();
        let first_path = path.split(';').find(|entry| !entry.trim().is_empty());
        Ok(Self::same_env_path(&home, &link.to_string_lossy())
            && first_path.is_some_and(|entry| Self::same_env_path(entry, &link.join("bin").to_string_lossy())))
    }

    fn updated_path(existing: &str, previous_home: &str, link: &Path) -> String {
        let stable_bin = link.join("bin").to_string_lossy().into_owned();
        let previous_bin = if previous_home.is_empty() {
            None
        } else {
            Some(Path::new(previous_home).join("bin").to_string_lossy().into_owned())
        };
        let mut entries: Vec<_> = existing.split(';')
            .filter(|entry| !entry.trim().is_empty())
            .filter(|entry| !Self::same_env_path(entry, &stable_bin)
                && !previous_bin.as_ref().is_some_and(|old| Self::same_env_path(entry, old)))
            .map(str::to_string)
            .collect();
        entries.insert(0, stable_bin);
        entries.join(";")
    }

    fn wide_bytes(value: &str) -> Vec<u8> {
        value.encode_utf16().chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes).collect()
    }

    fn install_registry(key: &RegKey, link: &Path) -> Result<()> {
        let old_path_raw = match key.get_raw_value("Path") {
            Ok(value) => Some(value),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(JdkError::EnvError(format!("Cannot read user PATH: {error}"))),
        };
        if old_path_raw.as_ref().is_some_and(|value| value.vtype != REG_SZ && value.vtype != REG_EXPAND_SZ) {
            return Err(JdkError::EnvError("User PATH is not a string registry value".to_string()));
        }
        let old_path: String = if old_path_raw.is_some() {
            key.get_value("Path").map_err(|e| JdkError::EnvError(format!("Cannot read user PATH: {e}")))?
        } else { String::new() };
        let previous_home: String = key.get_value("JAVA_HOME").unwrap_or_default();
        let new_path = Self::updated_path(&old_path, &previous_home, link);
        let new_path_raw = RegValue {
            bytes: Self::wide_bytes(&new_path),
            vtype: old_path_raw.as_ref().map(|value| value.vtype.clone()).unwrap_or(REG_EXPAND_SZ),
        };
        key.set_raw_value("Path", &new_path_raw)
            .map_err(|e| JdkError::EnvError(format!("Cannot update user PATH: {e}")))?;

        if let Err(e) = key.set_value("JAVA_HOME", &link.to_string_lossy().into_owned()) {
            let rollback = if let Some(old) = &old_path_raw {
                key.set_raw_value("Path", old)
            } else {
                key.delete_value("Path")
            };
            let detail = match rollback {
                Ok(()) => format!("Cannot update user JAVA_HOME: {e}"),
                Err(restore) => format!("Cannot update user JAVA_HOME: {e}; PATH restore also failed: {restore}"),
            };
            return Err(JdkError::EnvError(detail));
        }
        println!("[OK] User JAVA_HOME now points to: {}", link.display());
        println!("[OK] User PATH now starts with: {}", link.join("bin").display());
        Ok(())
    }

    fn switch_junction(link: &Path, target: &Path) -> Result<Option<PathBuf>> {
        let target = target.canonicalize()
            .map_err(|e| JdkError::InvalidPath(format!("{}: {e}", target.display())))?;
        let old_target = Self::existing_junction_target(link)?;
        if old_target.as_ref().is_some_and(|old| same_jdk_path(old, &target)) {
            return Ok(old_target);
        }
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| JdkError::EnvError(format!("Cannot make junction staging name: {e}")))?
            .as_nanos();
        let staging = link.with_file_name(format!(".jsh-current-{}-{nonce}", std::process::id()));
        if let Err(e) = junction::create(&target, &staging) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                let _ = fs::remove_dir(&staging);
            }
            return Err(JdkError::EnvError(format!("Cannot stage JDK junction {}: {e}", staging.display())));
        }
        if old_target.is_some() {
            if let Err(e) = Self::remove_junction(link) {
                let _ = Self::remove_junction(&staging);
                return Err(e);
            }
        }
        if let Err(e) = fs::rename(&staging, link) {
            let _ = Self::remove_junction(&staging);
            if let Some(old) = &old_target {
                if let Err(restore) = junction::create(old, link) {
                    return Err(JdkError::EnvError(format!(
                        "Cannot activate junction {}: {e}; restore also failed: {restore}", link.display()
                    )));
                }
            }
            return Err(JdkError::EnvError(format!("Cannot activate junction {}: {e}", link.display())));
        }
        Ok(old_target)
    }

    fn restore_junction(link: &Path, old_target: Option<&Path>) -> Result<()> {
        if let Some(old) = old_target {
            Self::switch_junction(link, old)?;
        } else if Self::existing_junction_target(link)?.is_some() {
            Self::remove_junction(link)?;
        }
        Ok(())
    }

    fn existing_junction_target(link: &Path) -> Result<Option<PathBuf>> {
        match fs::symlink_metadata(link) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(JdkError::EnvError(format!("Cannot inspect {}: {e}", link.display()))),
            Ok(_) => junction::get_target(link).map(Some).map_err(|e| JdkError::EnvError(format!(
                "{} already exists but is not a readable JDK junction: {e}", link.display()
            ))),
        }
    }

    fn remove_junction(link: &Path) -> Result<()> {
        junction::delete(link).map_err(|e| JdkError::EnvError(format!(
            "Cannot remove junction {}: {e}", link.display()
        )))?;
        // Deleting the reparse point leaves an ordinary empty directory.
        fs::remove_dir(link).map_err(|e| JdkError::EnvError(format!(
            "Cannot remove junction directory {}: {e}", link.display()
        )))
    }

    fn broadcast_environment_change() {
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        let environment: Vec<u16> = "Environment".encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let result = SendMessageTimeoutW(
                HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(environment.as_ptr() as isize),
                SMTO_ABORTIFHUNG, 5000, None,
            );
            if result.0 == 0 {
                eprintln!("Warning: Windows did not broadcast the environment change; restart your terminal once.");
            }
        }
    }

    /// A terminal that inherited the stable link for both variables will see
    /// subsequent junction changes without a restart.
    pub fn shell_uses_link(&self) -> Result<bool> {
        let link = Self::link_path()?;
        let home = std::env::var("JAVA_HOME").unwrap_or_default();
        if !Self::same_env_path(&home, &link.to_string_lossy()) { return Ok(false); }
        let stable_bin = link.join("bin");
        let path = std::env::var("PATH").unwrap_or_default();
        let first_java_dir = path.split(';')
            .map(|entry| entry.trim().trim_matches('"'))
            .find(|entry| Path::new(entry).join("java.exe").is_file());
        Ok(first_java_dir.is_some_and(|entry| Self::same_env_path(entry, &stable_bin.to_string_lossy())))
    }
}

impl EnvUpdater for WindowsEnvUpdater {
    fn update_java_home(&self, path: &Path) -> Result<()> {
        let link = Self::link_path()?;
        Self::ensure_switchable_location(&link)?;
        // Opening the user key for writing before the junction change makes a
        // permission failure leave the selected JDK untouched.
        let key = Self::environment_key()?;
        let needs_setup = !Self::registry_ready(&key, &link)?;
        let old_target = Self::switch_junction(&link, path)?;
        if needs_setup {
            if let Err(e) = Self::install_registry(&key, &link) {
                if let Err(restore) = Self::restore_junction(&link, old_target.as_deref()) {
                    return Err(JdkError::EnvError(format!("{e}; junction restore also failed: {restore}")));
                }
                return Err(e);
            }
            Self::broadcast_environment_change();
        }
        println!("[OK] Active JDK junction: {} -> {}", link.display(), path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn first_setup_updates_registry_values_without_touching_real_environment() {
        let name = format!("Software\\j-switch-test-{}", std::process::id());
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = root.create_subkey(&name).unwrap();
        key.set_value("Path", &"C:\\Tools\\bin;C:\\Java\\old\\bin").unwrap();
        key.set_value("JAVA_HOME", &"C:\\Java\\old").unwrap();
        let link = Path::new("C:\\jsh\\jsh-current");
        assert!(!WindowsEnvUpdater::registry_ready(&key, link).unwrap());
        WindowsEnvUpdater::install_registry(&key, link).unwrap();
        assert!(WindowsEnvUpdater::registry_ready(&key, link).unwrap());
        assert_eq!(key.get_value::<String, _>("Path").unwrap(),
            "C:\\jsh\\jsh-current\\bin;C:\\Tools\\bin");
        drop(key);
        root.delete_subkey(&name).unwrap();
    }

    #[test]
    fn first_setup_creates_a_missing_user_path() {
        let name = format!("Software\\j-switch-empty-test-{}", std::process::id());
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = root.create_subkey(&name).unwrap();
        let link = Path::new("C:\\jsh\\jsh-current");
        WindowsEnvUpdater::install_registry(&key, link).unwrap();
        assert_eq!(key.get_value::<String, _>("Path").unwrap(), "C:\\jsh\\jsh-current\\bin");
        assert_eq!(key.get_value::<String, _>("JAVA_HOME").unwrap(), "C:\\jsh\\jsh-current");
        drop(key);
        root.delete_subkey(&name).unwrap();
    }

    #[test]
    fn path_setup_preserves_unrelated_entries() {
        let link = Path::new("C:\\jsh\\jsh-current");
        let updated = WindowsEnvUpdater::updated_path(
            "C:\\Tools\\bin;C:\\Java\\old\\bin;C:\\Other\\jdk\\bin",
            "C:\\Java\\old", link,
        );
        assert_eq!(updated, "C:\\jsh\\jsh-current\\bin;C:\\Tools\\bin;C:\\Other\\jdk\\bin");
    }

    #[test]
    fn switching_junction_keeps_both_installations() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::write(first.join("marker"), b"first").unwrap();
        fs::write(second.join("marker"), b"second").unwrap();
        let link = temp.path().join("jsh-current");

        assert!(WindowsEnvUpdater::switch_junction(&link, &first).unwrap().is_none());
        assert_eq!(fs::read(link.join("marker")).unwrap(), b"first");
        let old = WindowsEnvUpdater::switch_junction(&link, &second).unwrap().unwrap();
        assert_eq!(fs::read(link.join("marker")).unwrap(), b"second");
        WindowsEnvUpdater::restore_junction(&link, Some(&old)).unwrap();
        assert_eq!(fs::read(link.join("marker")).unwrap(), b"first");
        assert!(first.join("marker").exists());
        assert!(second.join("marker").exists());
        WindowsEnvUpdater::restore_junction(&link, None).unwrap();
    }

    #[test]
    fn refuses_to_replace_a_regular_directory() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("jdk");
        let occupied = temp.path().join("jsh-current");
        fs::create_dir(&target).unwrap();
        fs::create_dir(&occupied).unwrap();
        fs::write(occupied.join("keep"), b"user data").unwrap();
        assert!(WindowsEnvUpdater::switch_junction(&occupied, &target).is_err());
        assert_eq!(fs::read(occupied.join("keep")).unwrap(), b"user data");
    }

    #[test]
    fn replaces_a_junction_whose_old_target_was_removed() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::write(second.join("marker"), b"second").unwrap();
        let link = temp.path().join("jsh-current");
        junction::create(&first, &link).unwrap();
        fs::remove_dir(&first).unwrap();
        WindowsEnvUpdater::switch_junction(&link, &second).unwrap();
        assert_eq!(fs::read(link.join("marker")).unwrap(), b"second");
    }

    #[test]
    fn an_existing_path_entry_uses_the_new_target_immediately() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("jdk-17");
        let second = temp.path().join("jdk-25");
        for (jdk, label) in [(&first, "17"), (&second, "25")] {
            fs::create_dir_all(jdk.join("bin")).unwrap();
            fs::write(jdk.join("bin").join("version.cmd"), format!("@echo {label}\r\n")).unwrap();
        }
        let link = temp.path().join("jsh-current");
        let stable_path = format!("{};{}", link.join("bin").display(), std::env::var("PATH").unwrap());
        let run = || {
            let output = Command::new("cmd.exe").args(["/C", "version.cmd"])
                .env("PATH", &stable_path).output().unwrap();
            assert!(output.status.success());
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        WindowsEnvUpdater::switch_junction(&link, &first).unwrap();
        assert_eq!(run(), "17");
        WindowsEnvUpdater::switch_junction(&link, &second).unwrap();
        assert_eq!(run(), "25");
    }
}
