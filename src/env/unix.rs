use crate::env::EnvUpdater;
use crate::error::{JdkError, Result};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct UnixEnvUpdater;

impl UnixEnvUpdater {
    pub fn new() -> Self {
        Self
    }

    fn get_shell_rc_path() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| JdkError::EnvError("Cannot find home directory".to_string()))?;

        // Try to detect shell
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Ok(home.join(".zshrc"));
            } else if shell.contains("bash") {
                return Ok(home.join(".bashrc"));
            }
        }

        // Default to .bashrc
        Ok(home.join(".bashrc"))
    }

    fn update_shell_rc(&self, java_home: &Path) -> Result<()> {
        let rc_path = Self::get_shell_rc_path()?;
        let java_home_str = java_home.to_str()
            .ok_or_else(|| JdkError::EnvError("JDK path is not valid UTF-8".to_string()))?;
        let quoted_home = Self::shell_quote(java_home_str);
        let write_path = if rc_path.exists() { rc_path.canonicalize()? } else { rc_path.clone() };

        // Read existing content
        let mut lines = Vec::new();
        let mut found_java_home = false;
        let mut found_path = false;

        if write_path.exists() {
            let file = std::fs::File::open(&write_path)
                .map_err(|e| JdkError::IoError(e))?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line.map_err(|e| JdkError::IoError(e))?;
                if line.contains("export JAVA_HOME=") && line.contains("# jsh managed") {
                    lines.push(format!("export JAVA_HOME={quoted_home}  # jsh managed"));
                    found_java_home = true;
                } else if line.contains("export PATH=") && line.contains("$JAVA_HOME/bin") && line.contains("# jsh managed") {
                    lines.push(format!("export PATH=\"$JAVA_HOME/bin:$PATH\"  # jsh managed"));
                    found_path = true;
                } else {
                    lines.push(line);
                }
            }
        }

        // Add new entries if not found
        if !found_java_home {
            lines.push("\n# jsh managed - do not edit manually".to_string());
            lines.push(format!("export JAVA_HOME={quoted_home}  # jsh managed"));
        }
        if !found_path {
            lines.push(format!("export PATH=\"$JAVA_HOME/bin:$PATH\"  # jsh managed"));
        }

        // Write back
        let parent = write_path.parent()
            .ok_or_else(|| JdkError::EnvError("Shell profile has no parent directory".to_string()))?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        if let Ok(metadata) = std::fs::metadata(&write_path) {
            file.as_file().set_permissions(metadata.permissions())?;
        }

        for line in lines {
            writeln!(file, "{}", line).map_err(|e| JdkError::IoError(e))?;
        }
        file.flush()?;
        file.as_file().sync_all()?;
        file.persist(&write_path)
            .map_err(|e| JdkError::EnvError(format!("Cannot replace shell profile: {}", e.error)))?;

        println!("[OK] Updated {}", rc_path.display());
        println!("  Please run: source {}", rc_path.display());

        Ok(())
    }

    fn shell_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

impl EnvUpdater for UnixEnvUpdater {
    fn update_java_home(&self, path: &Path) -> Result<()> {
        self.update_shell_rc(path)?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::UnixEnvUpdater;

    #[test]
    fn shell_path_does_not_expand_special_characters() {
        assert_eq!(UnixEnvUpdater::shell_quote("/tmp/$HOME/`test`/a'b"),
            "'/tmp/$HOME/`test`/a'\\''b'");
    }
}
