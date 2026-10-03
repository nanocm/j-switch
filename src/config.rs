use crate::error::{JdkError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JdkInfo {
    pub path: PathBuf,
    pub version: String,
    pub vendor: Option<String>,
    pub java_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub current_jdk: Option<String>,
    #[serde(default)]
    pub jdks: HashMap<String, JdkInfo>,
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
    #[serde(default)]
    pub scan_dirs: Vec<PathBuf>,
}

fn default_download_dir() -> PathBuf {
    PathBuf::from("downloads")
}

fn default_install_dir() -> PathBuf {
    PathBuf::from("jdks")
}

/// Use the canonical path when possible so alternate spellings of one install
/// do not create separate entries. Windows paths are case insensitive.
fn path_identity(path: &Path) -> String {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = resolved.to_string_lossy().replace('\\', "/");
    let text = text.trim_end_matches('/');
    if cfg!(windows) { text.to_lowercase() } else { text.to_string() }
}

pub fn same_jdk_path(a: &Path, b: &Path) -> bool {
    path_identity(a) == path_identity(b)
}

fn safe_id_part(value: &str) -> String {
    value.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' })
        .collect()
}

/// The full version is readable; the path hash distinguishes identical builds
/// installed in different directories without relying on scan order.
pub fn jdk_id(info: &JdkInfo) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path_identity(&info.path).as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let full_version = info.java_version.as_deref().unwrap_or("unknown");
    format!("{}-{}-{hash:016x}", safe_id_part(&info.version), safe_id_part(full_version))
}

impl Config {
    fn resolve_directory(config_dir: &Path, configured: &Path, key: &str) -> Result<PathBuf> {
        if configured.as_os_str().is_empty() {
            return Err(JdkError::ConfigError(format!("{key} cannot be empty")));
        }
        Ok(if configured.is_absolute() {
            configured.to_path_buf()
        } else {
            config_dir.join(configured)
        })
    }

    pub fn archive_dir(&self) -> Result<PathBuf> {
        Self::resolve_directory(&Self::config_dir()?, &self.download_dir, "download_dir")
    }

    pub fn install_dir(&self) -> Result<PathBuf> {
        Self::resolve_directory(&Self::config_dir()?, &self.install_dir, "install_dir")
    }

    /// Continue finding installations in the original directory after the
    /// user changes install_dir. Existing JDKs are not moved automatically.
    pub fn managed_install_dirs(&self) -> Result<Vec<PathBuf>> {
        let config_dir = Self::config_dir()?;
        self.managed_install_dirs_at(&config_dir)
    }

    fn managed_install_dirs_at(&self, config_dir: &Path) -> Result<Vec<PathBuf>> {
        let selected = Self::resolve_directory(config_dir, &self.install_dir, "install_dir")?;
        let legacy = config_dir.join("jdks");
        let mut dirs = vec![selected];
        if !same_jdk_path(&dirs[0], &legacy) {
            dirs.push(legacy);
        }
        Ok(dirs)
    }

    pub fn config_dir() -> Result<PathBuf> {
        let exe_path = std::env::current_exe()
            .map_err(|e| JdkError::ConfigError(format!("Cannot get executable path: {e}")))?;
        exe_path.parent().map(Path::to_path_buf)
            .ok_or_else(|| JdkError::ConfigError("Cannot get executable directory".to_string()))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() { return Ok(Self::default()); }
        let content = fs::read_to_string(&path)
            .map_err(|e| JdkError::ConfigError(format!("Failed to read config: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| JdkError::ConfigError(format!("Failed to parse config: {e}")))
    }

    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::config_path()?)
    }

    fn save_to_path(&self, path: &Path) -> Result<()> {
        let dir = path.parent().ok_or_else(|| JdkError::ConfigError(
            "Config path has no parent directory".to_string()
        ))?;
        fs::create_dir_all(&dir)
            .map_err(|e| JdkError::ConfigError(format!("Failed to create config dir: {e}")))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| JdkError::ConfigError(format!("Failed to serialize config: {e}")))?;
        let mut staged = tempfile::NamedTempFile::new_in(&dir)
            .map_err(|e| JdkError::ConfigError(format!("Failed to stage config: {e}")))?;
        staged.write_all(content.as_bytes())
            .and_then(|_| staged.flush())
            .and_then(|_| staged.as_file().sync_all())
            .map_err(|e| JdkError::ConfigError(format!("Failed to write staged config: {e}")))?;
        staged.persist(path)
            .map_err(|e| JdkError::ConfigError(format!("Failed to replace config: {}", e.error)))?;
        Ok(())
    }

    /// Keep the read-modify-write cycle in one process exclusive. Atomic save
    /// protects readers; this lock also prevents concurrent lost updates.
    pub fn lock_registry() -> Result<File> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .map_err(|e| JdkError::ConfigError(format!("Failed to create config dir: {e}")))?;
        let lock_path = dir.join("config.lock");
        let file = OpenOptions::new().read(true).write(true).create(true).open(lock_path)
            .map_err(|e| JdkError::ConfigError(format!("Failed to open config lock: {e}")))?;
        fs2::FileExt::lock_exclusive(&file)
            .map_err(|e| JdkError::ConfigError(format!("Failed to lock config: {e}")))?;
        Ok(file)
    }

    /// Migrate the former major-version keys without losing the active JDK.
    pub fn normalize_registry(&mut self) -> Result<bool> {
        let old_jdks = std::mem::take(&mut self.jdks);
        let old_current = self.current_jdk.clone();
        let mut new_current = None;
        let mut changed = false;
        for (old_key, info) in old_jdks {
            let key = jdk_id(&info);
            changed |= old_key != key;
            if self.jdks.get(&key).is_some_and(|other| !same_jdk_path(&other.path, &info.path)) {
                return Err(JdkError::ConfigError(format!("JDK identifier collision: {key}")));
            }
            if old_current.as_deref() == Some(&old_key) { new_current = Some(key.clone()); }
            changed |= self.jdks.insert(key, info).is_some();
        }
        if new_current.is_some() { self.current_jdk = new_current; }
        Ok(changed || self.current_jdk != old_current)
    }

    /// Register or refresh an installation by path. A shared major version
    /// never causes one installation to replace another.
    pub fn register_jdk(&mut self, info: JdkInfo) -> Result<(String, bool)> {
        let key = jdk_id(&info);
        if self.jdks.get(&key).is_some_and(|other| !same_jdk_path(&other.path, &info.path)) {
            return Err(JdkError::ConfigError(format!("JDK identifier collision: {key}")));
        }
        let obsolete: Vec<_> = self.jdks.iter()
            .filter(|(existing_key, existing)| *existing_key != &key && same_jdk_path(&existing.path, &info.path))
            .map(|(existing_key, _)| existing_key.clone())
            .collect();
        for old_key in &obsolete {
            self.jdks.remove(old_key);
            if self.current_jdk.as_deref() == Some(old_key) { self.current_jdk = Some(key.clone()); }
        }
        let changed = !obsolete.is_empty() || self.jdks.get(&key) != Some(&info);
        self.jdks.insert(key.clone(), info);
        Ok((key, changed))
    }

    pub fn get_current(&self) -> Option<&JdkInfo> {
        self.current_jdk.as_ref().and_then(|key| self.jdks.get(key))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            current_jdk: None,
            jdks: HashMap::new(),
            download_dir: default_download_dir(),
            install_dir: default_install_dir(),
            scan_dirs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(path: &Path, full_version: &str) -> JdkInfo {
        JdkInfo {
            path: path.to_path_buf(),
            version: "17".to_string(),
            vendor: Some("Eclipse Temurin".to_string()),
            java_version: Some(full_version.to_string()),
        }
    }

    #[test]
    fn same_major_and_identical_builds_keep_distinct_ids() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first");
        let second = dir.path().join("second");
        let third = dir.path().join("third");
        for path in [&first, &second, &third] { fs::create_dir(path).unwrap(); }
        let mut config = Config::default();
        let (first_id, _) = config.register_jdk(info(&first, "17.0.10")).unwrap();
        let (second_id, _) = config.register_jdk(info(&second, "17.0.11")).unwrap();
        let (third_id, _) = config.register_jdk(info(&third, "17.0.10")).unwrap();
        assert_eq!(config.jdks.len(), 3);
        assert_ne!(first_id, second_id);
        assert_ne!(first_id, third_id);
        assert_eq!(config.register_jdk(info(&first, "17.0.10")).unwrap(), (first_id, false));
    }

    #[test]
    fn legacy_major_key_migrates_active_selection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jdk");
        fs::create_dir(&path).unwrap();
        let mut config = Config::default();
        config.jdks.insert("17".to_string(), info(&path, "17.0.10"));
        config.current_jdk = Some("17".to_string());
        assert!(config.normalize_registry().unwrap());
        let id = config.current_jdk.as_ref().unwrap();
        assert_ne!(id, "17");
        assert_eq!(config.jdks[id].path, path);
        assert!(!config.normalize_registry().unwrap());
    }

    #[test]
    fn scan_dirs_loads_from_config_without_breaking_old_files() {
        let legacy = r#"{"current_jdk":null,"jdks":{},"download_dir":"downloads"}"#;
        let config: Config = serde_json::from_str(legacy).unwrap();
        assert!(config.scan_dirs.is_empty());
        assert_eq!(config.install_dir, PathBuf::from("jdks"));

        let minimal: Config = serde_json::from_str(r#"{"install_dir":"other-jdks"}"#).unwrap();
        assert!(minimal.jdks.is_empty());
        assert!(minimal.current_jdk.is_none());
        assert_eq!(minimal.download_dir, PathBuf::from("downloads"));
        assert_eq!(minimal.install_dir, PathBuf::from("other-jdks"));

        let customized = r#"{"current_jdk":null,"jdks":{},"download_dir":"downloads","install_dir":"E:\\Java","scan_dirs":["D:\\Java","E:\\SDKs"]}"#;
        let config: Config = serde_json::from_str(customized).unwrap();
        assert_eq!(config.install_dir, PathBuf::from("E:\\Java"));
        assert_eq!(config.scan_dirs, vec![PathBuf::from("D:\\Java"), PathBuf::from("E:\\SDKs")]);
    }

    #[test]
    fn custom_install_dir_keeps_legacy_installs_discoverable() {
        let base = Path::new("base");
        let mut config = Config::default();
        assert_eq!(
            config.managed_install_dirs_at(base).unwrap(),
            vec![base.join("jdks")]
        );

        config.install_dir = PathBuf::from("other-jdks");
        assert_eq!(
            config.managed_install_dirs_at(base).unwrap(),
            vec![base.join("other-jdks"), base.join("jdks")]
        );

        let absolute = std::env::current_dir().unwrap().join("external-jdks");
        config.install_dir = absolute.clone();
        assert_eq!(
            config.managed_install_dirs_at(base).unwrap(),
            vec![absolute, base.join("jdks")]
        );

        config.install_dir = PathBuf::new();
        assert!(config.managed_install_dirs_at(base).is_err());
        assert_eq!(
            Config::resolve_directory(base, Path::new("cache"), "download_dir").unwrap(),
            base.join("cache")
        );
    }

    #[test]
    fn atomic_save_replaces_an_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut config = Config::default();
        config.save_to_path(&path).unwrap();
        config.scan_dirs.push(PathBuf::from("custom"));
        config.save_to_path(&path).unwrap();
        let loaded: Config = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.scan_dirs, vec![PathBuf::from("custom")]);
    }
}
