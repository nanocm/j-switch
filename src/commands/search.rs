use crate::downloader::adoptium::AdoptiumSource;
use crate::downloader::traits::JdkSource;
use crate::error::Result;
use colored::Colorize;
use std::collections::HashMap;

pub async fn search_command(keyword: Option<String>) -> Result<()> {
    println!("{}", "Searching for available JDK versions...".cyan());

    let source = AdoptiumSource::new()?;
    let mut packages = source.fetch_version().await?;

    println!(
        "{}",
        format!("Found {} packages from {}", packages.len(), source.name()).bright_black()
    );

    if let Some(keyword) = keyword {
        let keyword_lower = keyword.to_lowercase();

        packages.retain(|p| {
            p.version.to_lowercase().contains(&keyword_lower)
                || p.vendor.to_lowercase().contains(&keyword_lower)
                || p.major_version.to_string() == keyword
        });
        println!(
            "{}",
            format!(
                "Filtered to {} packages matching '{}'",
                packages.len(),
                keyword
            )
            .bright_black()
        );
        if packages.is_empty() {
            println!("\n{}", "No matching JDK versions found.".yellow());
            println!("Try searching without a keyword or with a different term.");
            return Ok(());
        }
    }

    let mut grouped: HashMap<u32, Vec<_>> = HashMap::new();
    for pkg in packages {
        grouped
            .entry(pkg.major_version)
            .or_insert_with(Vec::new)
            .push(pkg);
    }
    println!("\n{}", "Available JDK versions:".bold());
    println!("{}", "=".repeat(80).bright_black());

    let mut versions: Vec<_> = grouped.keys().collect();
    versions.sort_by(|a, b| b.cmp(a));

    for &version in &versions  {
        let pkgs = &grouped[version];
        let is_lts = pkgs[0].is_lts;
        // version number title
        let version_text = format!("JDK {}", version);
        let lts_tag = if is_lts {
            "(LTS)".green().bold()
        } else {
            "".normal()
        };
        println!("\n  {}{}", version_text.bold().cyan(), lts_tag);

        for pkg in pkgs {
            let size_mb = pkg.size / 1024 / 1024;
            println!("    └─ {:8} {} {:>4}",
                     pkg.vendor.bright_black(),
                     pkg.version.white(),
                     format!("[{} MB]", size_mb).bright_black()
            );
        }
    }

    println!("\n{}", "-".repeat(80).bright_black());
    println!("Total: {} version(s)", versions.len());
    println!("\nUse: {} to download and install",
             "jsh download <version>".green());

    Ok(())
}
