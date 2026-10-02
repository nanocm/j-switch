use crate::config::Config;
use crate::downloader::adoptium::AdoptiumSource;
use crate::downloader::downloader::Downloader;
use crate::downloader::extractor::Extractor;
use crate::downloader::progress::ProgressDisplay;
use crate::downloader::traits::{JdkPackage, JdkSource};
use crate::error::{JdkError, Result};
use crate::jdk::JdkManager;
use crate::jdk::detector::JdkDetector;
use colored::Colorize;
use std::path::Path;

pub async fn download_command(version: &str, vendor: &str) -> Result<()> {
    println!("  Version: {version}");
    println!("  Vendor: {vendor}");
    println!("{}", format!("Searching for JDK {version}...").cyan());

    let source: Box<dyn JdkSource> = match vendor.to_lowercase().as_str() {
        "temurin" | "adoptium" => Box::new(AdoptiumSource::new()),
        _ => {
            return Err(JdkError::DownloadError(format!(
                "Unsupported vendor '{vendor}'. Available vendor: temurin"
            )));
        }
    };
    let version_num: u32 = version.parse()
        .map_err(|_| JdkError::InvalidVersion(version.to_string()))?;
    let package = source.find_package(version_num).await?;

    println!("\n{}", "Found package:".green().bold());
    println!("  Version:     {}", package.version);
    println!("  Vendor:      {}", package.vendor);
    println!("  Size:        {} MB", package.size / 1024 / 1024);
    println!("  Platform:    {} ({})", package.os, package.arch);
    println!("  File type:   {}", package.file_type);
    if package.is_lts { println!("  Support:     {}", "LTS (Long Term Support)".green()); }

    let extractor = Extractor::new();
    let install_base = Config::config_dir()?.join("jdks");
    std::fs::create_dir_all(&install_base)?;
    let release_name = release_name(&package);
    let install_dir = install_base.join(&release_name);

    let jdk_path = if install_dir.exists() {
        let path = extractor.find_jdk_root(&install_dir)?;
        verify_installed_jdk(&path, &package)?;
        println!("{}", format!("Using installed JDK: {}", path.display()).green());
        path
    } else {
        let filename = format!("{release_name}.{}", package.file_type);
        println!("\n{}", "Downloading...".cyan());
        let downloader = Downloader::new()?;
        let archive_path = downloader.download_file(
            &package.download_url,
            &filename,
            package.size,
            package.checksum.as_deref(),
            ProgressDisplay::simple_callback(),
        ).await?;
        println!("{}", "[OK] Download complete".green());

        println!("\n{}", "Extracting...".cyan());
        // A failed extraction leaves only a temporary directory. An existing
        // installation is never overwritten by files from another archive.
        let staging = tempfile::Builder::new().prefix(".jsh-staging-").tempdir_in(&install_base)?;
        let staged_root = extractor.extract(&archive_path, staging.path())?;
        verify_installed_jdk(&staged_root, &package)?;
        let relative_root = staged_root.strip_prefix(staging.path())
            .map_err(|e| JdkError::ExtractionError(e.to_string()))?;
        let installed_root = install_dir.join(relative_root);
        std::fs::rename(staging.path(), &install_dir)?;
        println!("{}", format!("[OK] Extracted to: {}", installed_root.display()).green());
        if let Err(e) = std::fs::remove_file(&archive_path) {
            println!("{}", format!("Warning: Failed to remove archive: {e}").yellow());
        }
        installed_root
    };

    let mut manager = JdkManager::new()?;
    let id = manager.register_jdk_path(&jdk_path)?;
    println!("\n{} {}", "[SUCCESS]".green().bold(), "JDK installed and registered!".green());
    println!("  ID: {id}");
    println!("\n{}", "Next steps:".bold());
    println!("  1. List all JDKs:    {}", "jsh list".cyan());
    println!("  2. Activate this JDK: {}", format!("jsh use {id}").cyan());
    Ok(())
}

fn safe_component(value: &str) -> String {
    value.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' })
        .collect()
}

fn release_name(package: &JdkPackage) -> String {
    format!("jdk-{}-{}-{}-{}",
        safe_component(&package.vendor), safe_component(&package.version),
        safe_component(&package.os), safe_component(&package.arch))
}

fn verify_installed_jdk(path: &Path, package: &JdkPackage) -> Result<()> {
    if !JdkDetector::is_valid_jdk(path) {
        return Err(JdkError::InvalidPath(path.display().to_string()));
    }
    let actual = JdkDetector::get_jdk_info(path)
        .ok_or_else(|| JdkError::InvalidPath(path.display().to_string()))?;
    if actual.version != package.major_version.to_string() {
        return Err(JdkError::ExtractionError(format!(
            "Expected JDK {}, found JDK {} at {}",
            package.major_version, actual.version, path.display()
        )));
    }
    Ok(())
}
