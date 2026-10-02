use crate::error::{JdkError, Result};
use crate::config::same_jdk_path;
use crate::jdk::JdkManager;
use colored::*;

pub fn current_command() -> Result<()> {
    let manager = JdkManager::new()?;
    let java_home_env = std::env::var("JAVA_HOME").ok();

    // Priority 1: Check JAVA_HOME environment variable first
    let (current, version_key, source) = if let Some(ref java_home) = java_home_env {
        let java_home_path = std::path::PathBuf::from(java_home);
        
        // Find JDK info by path from registered JDKs
        let found = manager.list_jdks()
            .into_iter()
            .find(|(_, info)| same_jdk_path(&info.path, &java_home_path));
        
        if let Some((key, info)) = found {
            (info, key.clone(), "environment variable")
        } else {
            // JAVA_HOME is set but not managed by jsh
            return Err(JdkError::ConfigError(format!(
                "JAVA_HOME is set to '{}' but this JDK is not managed by jsh.\nRun 'jsh list' to see available JDKs.",
                java_home
            )));
        }
    } else {
        // Priority 2: Fall back to config file
        let current = manager.get_current()
            .ok_or(JdkError::NoActiveJdk)?;
        let version_key = manager.get_current_version().unwrap().clone();
        (current, version_key, "config file (JAVA_HOME not set)")
    };

    println!("{}", "Current JDK:".bold());
    println!("{}", "=".repeat(60).bright_black());
    println!("{} {}", "Version:".bright_black(), format!("JDK {}", current.version).green().bold());
    println!("{} {}", "ID:".bright_black(), version_key);
    println!("{} {}", "Full Version:".bright_black(),
        current.java_version.as_deref().unwrap_or("unknown"));

    if let Some(vendor) = &current.vendor {
        println!("{} {}", "Vendor:".bright_black(), vendor);
    }
    
    println!("{} {}", "Path:".bright_black(), current.path.display());
    println!("{} {}", "Source:".bright_black(), source.bright_black());
    println!("{}", "=".repeat(60).bright_black());

    // Check environment status
    if java_home_env.is_some() {
        if source == "environment variable" {
            println!("\n{} JAVA_HOME is correctly set", "[OK]".green());
        } else {
            println!("\n{} JAVA_HOME is not set in environment", "[!]".yellow());
            println!("  The above JDK is from config file.");
            println!("  {}", "Try restarting your terminal or running the use command again.".yellow());
        }
    } else {
        println!("\n{} JAVA_HOME is not set in environment", "[!]".yellow());
    }
    
    Ok(())
}
