use crate::config::{JdkInfo, same_jdk_path};
use crate::error::{JdkError, Result};
use crate::jdk::JdkManager;
use crate::jdk::detector::JdkDetector;
use colored::*;
use std::path::PathBuf;

pub fn current_command() -> Result<()> {
    let manager = JdkManager::new()?;
    let java_home_env = std::env::var("JAVA_HOME").ok();

    let (current, id, source): (JdkInfo, Option<String>, &str) =
        if let Some(ref java_home) = java_home_env {
            let path = PathBuf::from(java_home);
            if let Some((key, info)) = manager.list_jdks().into_iter()
                .find(|(_, info)| same_jdk_path(&info.path, &path)) {
                (info.clone(), Some(key.clone()), "JAVA_HOME")
            } else if JdkDetector::is_valid_jdk(&path) {
                let info = JdkDetector::get_jdk_info(&path)
                    .ok_or_else(|| JdkError::InvalidPath(java_home.clone()))?;
                (info, None, "JAVA_HOME (unregistered)")
            } else {
                return Err(JdkError::InvalidPath(format!("JAVA_HOME points to an invalid JDK: {java_home}")));
            }
        } else {
            let info = manager.get_current().ok_or(JdkError::NoActiveJdk)?;
            if !JdkDetector::is_valid_jdk(&info.path) {
                return Err(JdkError::InvalidPath(format!(
                    "Configured JDK is unavailable: {}", info.path.display()
                )));
            }
            (info.clone(), manager.get_current_version().cloned(), "config (JAVA_HOME unset)")
        };

    println!("{}", "Current JDK:".bold());
    println!("{}", "=".repeat(60).bright_black());
    println!("{} {}", "Version:".bright_black(), format!("JDK {}", current.version).green().bold());
    println!("{} {}", "ID:".bright_black(), id.as_deref().unwrap_or("unregistered"));
    println!("{} {}", "Full Version:".bright_black(),
        current.java_version.as_deref().unwrap_or("unknown"));
    if let Some(vendor) = &current.vendor {
        println!("{} {}", "Vendor:".bright_black(), vendor);
    }
    println!("{} {}", "Path:".bright_black(), current.path.display());
    println!("{} {}", "Source:".bright_black(), source.bright_black());
    println!("{}", "=".repeat(60).bright_black());

    if java_home_env.is_none() {
        println!("\n{} JAVA_HOME is not set in this terminal", "[!]".yellow());
    } else if id.is_none() {
        println!("\n{} This JDK is not registered; run jsh list to add it.", "[!]".yellow());
    } else {
        println!("\n{} JAVA_HOME points to this JDK", "[OK]".green());
    }

    if let Some(actual_java) = first_java_on_path() {
        let expected_java = current.path.join("bin").join(java_executable_name());
        if !same_jdk_path(&actual_java, &expected_java) {
            println!("{} java on PATH resolves to {}", "[!]".yellow(), actual_java.display());
            println!("  Put the selected JDK's bin directory before other Java entries in PATH.");
        }
    }
    Ok(())
}

fn java_executable_name() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

fn first_java_on_path() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(java_executable_name()))
        .find(|candidate| candidate.is_file())
}
