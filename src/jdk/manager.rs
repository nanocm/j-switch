use crate::config::{Config, JdkInfo, same_jdk_path};
use crate::error::{JdkError, Result};
use crate::jdk::detector::JdkDetector;
use std::path::Path;
use std::fs::File;

pub struct JdkManager {
    config: Config,
    _registry_lock: Option<File>,
}

impl JdkManager {
    pub fn new() -> Result<Self> {
        let lock = Config::lock_registry()?;
        let (mut config, from_legacy) = Config::load_with_source()?;
        let normalized = config.normalize_registry()?;
        if from_legacy || normalized { config.save()?; }
        Ok(Self { config, _registry_lock: Some(lock) })
    }

    /// Refresh managed installs and JAVA_HOME. A system scan is explicit
    /// because walking drive roots makes an ordinary list unnecessarily slow.
    pub fn scan_jdks(&mut self, scan_system: bool) -> Result<Vec<JdkInfo>> {
        let config_dir = Config::config_dir()?;
        let mut search_dirs = self.config.managed_install_dirs()?;
        for dir in &self.config.scan_dirs {
            let resolved = if dir.is_absolute() { dir.clone() } else { config_dir.join(dir) };
            if !search_dirs.iter().any(|existing| same_jdk_path(existing, &resolved)) {
                if !resolved.is_dir() {
                    eprintln!("Warning: configured scan directory does not exist: {}", resolved.display());
                    continue;
                }
                search_dirs.push(resolved);
            }
        }
        let mut detected = Vec::new();
        for dir in &search_dirs {
            detected.extend(JdkDetector::scan_directory(dir)?);
        }
        if let Some(jdk) = JdkDetector::detect_java_home() {
            detected.push(jdk);
        }
        if scan_system {
            detected.extend(JdkDetector::detect_system_installations()?);
        }
        detected.sort_by(|a, b| a.path.cmp(&b.path));
        detected.dedup_by(|a, b| same_jdk_path(&a.path, &b.path));

        let mut changed = false;
        for jdk in &detected {
            changed |= self.config.register_jdk(jdk.clone())?.1;
        }
        if changed { self.config.save()?; }
        Ok(detected)
    }

    /// Register the exact directory returned by extraction, even when it is
    /// deeper than the system scan limit.
    pub fn register_jdk_path(&mut self, path: &Path) -> Result<String> {
        if !JdkDetector::is_valid_jdk(path) {
            return Err(JdkError::InvalidPath(path.display().to_string()));
        }
        let info = JdkDetector::get_jdk_info(path)
            .ok_or_else(|| JdkError::InvalidPath(path.display().to_string()))?;
        let (key, changed) = self.config.register_jdk(info)?;
        if changed { self.config.save()?; }
        Ok(key)
    }

    pub fn list_jdks(&self) -> Vec<(&String, &JdkInfo)> {
        let mut jdks: Vec<_> = self.config.jdks.iter()
            .filter(|(_, info)| JdkDetector::is_valid_jdk(&info.path))
            .collect();
        jdks.sort_by(|a, b| {
            let a_major = a.1.version.parse::<u32>().unwrap_or(0);
            let b_major = b.1.version.parse::<u32>().unwrap_or(0);
            a_major.cmp(&b_major).then_with(|| a.0.cmp(b.0))
        });
        jdks
    }

    pub fn unavailable_jdks(&self) -> Vec<(&String, &JdkInfo)> {
        let mut unavailable: Vec<_> = self.config.jdks.iter()
            .filter(|(_, info)| !JdkDetector::is_valid_jdk(&info.path))
            .collect();
        unavailable.sort_by(|a, b| a.0.cmp(b.0));
        unavailable
    }

    /// Removal is explicit because a JDK on a disconnected drive may return.
    pub fn prune_unavailable(&mut self) -> Result<usize> {
        let removed = self.prune_unavailable_in_memory();
        if removed > 0 { self.config.save()?; }
        Ok(removed)
    }

    fn prune_unavailable_in_memory(&mut self) -> usize {
        let before = self.config.jdks.len();
        self.config.jdks.retain(|_, info| JdkDetector::is_valid_jdk(&info.path));
        let removed = before - self.config.jdks.len();
        if removed > 0 {
            if self.config.current_jdk.as_ref().is_some_and(|key| !self.config.jdks.contains_key(key)) {
                self.config.current_jdk = None;
            }
        }
        removed
    }

    pub fn get_current(&self) -> Option<&JdkInfo> { self.config.get_current() }
    pub fn get_current_version(&self) -> Option<&String> { self.config.current_jdk.as_ref() }

    /// Exact IDs always work. A major or full version works when it identifies
    /// exactly one registered installation.
    pub fn resolve_jdk(&self, selector: &str) -> Result<(String, JdkInfo)> {
        if let Some(info) = self.config.jdks.get(selector) {
            return Self::validate_jdk(selector, info);
        }
        let matches: Vec<_> = self.config.jdks.iter()
            .filter(|(_, info)| JdkDetector::is_valid_jdk(&info.path)
                && (info.version == selector || info.java_version.as_deref() == Some(selector)))
            .collect();
        match matches.as_slice() {
            [] => Err(JdkError::JdkNotFound(selector.to_string())),
            [(key, info)] => Self::validate_jdk(key, info),
            _ => {
                let mut ids: Vec<_> = matches.iter().map(|(key, _)| (*key).clone()).collect();
                ids.sort();
                Err(JdkError::AmbiguousJdk(selector.to_string(), ids.join(", ")))
            }
        }
    }

    fn validate_jdk(key: &str, info: &JdkInfo) -> Result<(String, JdkInfo)> {
        if !JdkDetector::is_valid_jdk(&info.path) {
            return Err(JdkError::InvalidPath(format!("JDK path no longer valid: {}", info.path.display())));
        }
        Ok((key.to_string(), info.clone()))
    }

    pub fn set_current(&mut self, key: String) -> Result<()> {
        self.set_current_option(Some(key))
    }

    pub fn set_current_option(&mut self, key: Option<String>) -> Result<()> {
        if let Some(ref key) = key {
            if !self.config.jdks.contains_key(key) { return Err(JdkError::JdkNotFound(key.clone())); }
        }
        self.config.current_jdk = key;
        self.config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fake_jdk(base: &Path, name: &str, full_version: &str) -> JdkInfo {
        let path = base.join(name);
        fs::create_dir_all(path.join("bin")).unwrap();
        fs::create_dir_all(path.join("lib")).unwrap();
        let executable = if cfg!(windows) { "java.exe" } else { "java" };
        fs::write(path.join("bin").join(executable), b"").unwrap();
        let compiler = if cfg!(windows) { "javac.exe" } else { "javac" };
        fs::write(path.join("bin").join(compiler), b"").unwrap();
        JdkInfo {
            path,
            version: "17".to_string(),
            vendor: Some("Eclipse Temurin".to_string()),
            java_version: Some(full_version.to_string()),
        }
    }

    #[test]
    fn use_requires_id_when_version_is_ambiguous() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        let (first_id, _) = config.register_jdk(fake_jdk(dir.path(), "one", "17.0.10")).unwrap();
        let (second_id, _) = config.register_jdk(fake_jdk(dir.path(), "two", "17.0.11")).unwrap();
        config.register_jdk(fake_jdk(dir.path(), "three", "17.0.10")).unwrap();
        let manager = JdkManager { config, _registry_lock: None };

        assert!(matches!(manager.resolve_jdk("17"), Err(JdkError::AmbiguousJdk(_, _))));
        assert!(matches!(manager.resolve_jdk("17.0.10"), Err(JdkError::AmbiguousJdk(_, _))));
        assert_eq!(manager.resolve_jdk("17.0.11").unwrap().0, second_id);
        assert_eq!(manager.resolve_jdk(&first_id).unwrap().1.path, dir.path().join("one"));
    }

    #[test]
    fn missing_registration_does_not_make_a_version_ambiguous() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        let (valid_id, _) = config.register_jdk(fake_jdk(dir.path(), "valid", "17.0.11")).unwrap();
        let missing = fake_jdk(dir.path(), "missing", "17.0.10");
        let (missing_id, _) = config.register_jdk(missing.clone()).unwrap();
        fs::remove_dir_all(&missing.path).unwrap();
        let mut manager = JdkManager { config, _registry_lock: None };
        assert_eq!(manager.resolve_jdk("17").unwrap().0, valid_id);
        assert_eq!(manager.unavailable_jdks().len(), 1);
        manager.config.current_jdk = Some(missing_id);
        assert_eq!(manager.prune_unavailable_in_memory(), 1);
        assert!(manager.config.current_jdk.is_none());
        assert_eq!(manager.list_jdks().len(), 1);
    }
}
