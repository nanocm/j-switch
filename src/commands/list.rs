use crate::error::Result;
use crate::config::same_jdk_path;
use crate::jdk::JdkManager;
use colored::*;
use std::collections::HashMap;

pub fn list_command(scan_system: bool, prune: bool) -> Result<()> {
    let mut manager = JdkManager::new()?;

    if scan_system {
        println!("{}", "Scanning managed, configured, and system locations...".cyan());
    } else {
        println!("{}", "Checking registered JDKs, managed downloads, configured directories, and JAVA_HOME...".cyan());
    }
    manager.scan_jdks(scan_system)?;
    if prune {
        let removed = manager.prune_unavailable()?;
        println!("Removed {removed} unavailable JDK registration(s).");
    }
    let java_home_path = std::env::var("JAVA_HOME").ok().map(|s| std::path::PathBuf::from(s));

    let jdks = manager.list_jdks();
    let unavailable = manager.unavailable_jdks();
    let current_version = manager.get_current_version();
    let mut major_counts = HashMap::new();
    for (_, info) in &jdks {
        *major_counts.entry(info.version.as_str()).or_insert(0_usize) += 1;
    }
    
    if jdks.is_empty() {
        println!("{}", "No available JDK installations found.".yellow());
        print_unavailable(&unavailable);
        if !scan_system {
            println!("Add scan_dirs to config.json or run {} to discover other JDKs.", "jsh list --scan".green());
        }
        return Ok(());
    }
    
    println!("\n{}", "Installed JDKs:".bold()); 
    println!("{}", "=".repeat(80).bright_black()); 
    
    for (key, info) in &jdks {
        let is_current_in_config = current_version.map(|v| v == *key).unwrap_or(false);
        let is_current_in_env = java_home_path.as_ref()
            .map(|p| same_jdk_path(p, &info.path))
            .unwrap_or(false);
        
        // Priority: environment variable takes precedence
        let is_current = is_current_in_env || (is_current_in_config && java_home_path.is_none());
        
        let marker = if is_current { "*".green() } else { "-".bright_black() };
        
        let status_text = if is_current_in_env && is_current_in_config {
            "(active)".green()
        } else if is_current_in_env {
            "(active - environment only)".yellow()
        } else if is_current_in_config && java_home_path.is_some() {
            // Config says this is current, but env says otherwise
            "(config mismatch)".red()
        } else if is_current_in_config {
            "(config - env not set)".yellow()
        } else {
            "".normal()
        };
        
        println!("{} {} {}", 
            marker,
            format!("JDK {}", info.version).bold(),
            status_text
        );

        println!("  {} {}", "ID:".bright_black(), key);
        let selector = if major_counts[info.version.as_str()] == 1 { &info.version } else { key };
        println!("  {} jsh use {}", "Use:".bright_black(), selector);
        
        println!("  {} {}", "Version:".bright_black(),
            info.java_version.as_deref().unwrap_or("unknown"));
        
        if let Some(vendor) = &info.vendor {
            println!("  {} {}", "Vendor:".bright_black(), vendor);
        }
        
        println!("  {} {}", "Path:".bright_black(), info.path.display());
        println!();
    }
    
    println!("{}", "-".repeat(80).bright_black());
    println!("Total: {} JDK(s)", jdks.len());
    print_unavailable(&unavailable);

    if current_version.is_some_and(|key| unavailable.iter().any(|(id, _)| *id == key)) {
        println!("{}", "The configured current JDK is unavailable; choose another with jsh use.".yellow());
    }
    
    let has_active_in_env = jdks.iter().any(|(_, info)| {
        java_home_path.as_ref().map(|p| same_jdk_path(p, &info.path)).unwrap_or(false)
    });
    
    let has_config_mismatch = if let (Some(env_path), Some(config_ver)) = (&java_home_path, current_version) {
        jdks.iter()
            .find(|(k, _)| k == &config_ver)
            .map(|(_, info)| !same_jdk_path(&info.path, env_path))
            .unwrap_or(false)
    } else {
        false
    };
    
    if !has_active_in_env && java_home_path.is_none() && current_version.is_none() {
        println!("\n{}", "No JDK is currently active.".yellow());
        println!("Use {} to activate a JDK.", "jsh use <version>".green());
    } else if has_config_mismatch {
        println!("\n{}", "Warning:".yellow().bold());
        println!("  JAVA_HOME environment variable does not match jsh config.");
        println!("  Current JDK is determined by JAVA_HOME (shown with * above).");
        println!("  Run {} to sync config with environment.", "jsh use <version>".green());
    } else if java_home_path.is_some() && current_version.is_none() {
        println!("\n{}", "Tip:".cyan().bold());
        println!("  Your JAVA_HOME is set, but not managed by jsh.");
        println!("  Run {} to let jsh manage it.", "jsh use <version>".green());
    } else if java_home_path.is_none() && current_version.is_some() {
        println!("\n{}", "Warning:".yellow().bold());
        println!("  JAVA_HOME is not set in your environment.");
        println!("  Run {} again to set environment variables.", "jsh use <version>".green());
    }
    
    Ok(())
}

fn print_unavailable(unavailable: &[(&String, &crate::config::JdkInfo)]) {
    if unavailable.is_empty() { return; }
    println!("\n{}", format!("{} unavailable registration(s):", unavailable.len()).yellow());
    for (id, info) in unavailable {
        println!("  {id}: {}", info.path.display());
    }
    println!("Run {} to remove them.", "jsh list --prune".green());
}
