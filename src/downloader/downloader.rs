use crate::config::Config;
use crate::error::{JdkError, Result};
use futures_util::StreamExt;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
    download_dir: PathBuf,
}

impl Downloader {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let download_dir = Self::resolve_download_dir(&Config::config_dir()?, &config.download_dir)?;
        std::fs::create_dir_all(&download_dir)?;
        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .map_err(|e| JdkError::DownloadError(e.to_string()))?,
            download_dir,
        })
    }

    fn resolve_download_dir(config_dir: &Path, configured: &Path) -> Result<PathBuf> {
        if configured.as_os_str().is_empty() {
            return Err(JdkError::ConfigError("download_dir cannot be empty".to_string()));
        }
        Ok(if configured.is_absolute() { configured.to_path_buf() } else { config_dir.join(configured) })
    }

    pub async fn download_file<F>(
        &self,
        url: &str,
        filename: &str,
        expected_size: u64,
        checksum: Option<&str>,
        on_progress: F,
    ) -> Result<PathBuf>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        if Path::new(filename).file_name().and_then(|s| s.to_str()) != Some(filename) {
            return Err(JdkError::DownloadError("Invalid archive filename".to_string()));
        }
        let target_path = self.download_dir.join(filename);
        if Self::verify_file(&target_path, expected_size, checksum)? {
            println!("Using verified cached file: {}", target_path.display());
            return Ok(target_path);
        }

        let response = self.client.get(url).send().await
            .map_err(|e| JdkError::NetworkError(e.to_string()))?
            .error_for_status()
            .map_err(|e| JdkError::NetworkError(e.to_string()))?;
        let part_path = tempfile::Builder::new().prefix(".jsh-download-")
            .suffix(".part").tempfile_in(&self.download_dir)?.into_temp_path();
        let mut file = File::create(&part_path).await?;
        let mut downloaded = 0_u64;
        let mut hasher = Sha256::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => {
                    drop(file);
                    return Err(JdkError::NetworkError(e.to_string()));
                }
            };
            if let Err(e) = file.write_all(&chunk).await {
                drop(file);
                return Err(JdkError::IoError(e));
            }
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;
            on_progress(downloaded, expected_size);
        }
        file.flush().await?;
        drop(file);

        let actual_checksum = format!("{:x}", hasher.finalize());
        if downloaded != expected_size || checksum.is_some_and(|value| !actual_checksum.eq_ignore_ascii_case(value)) {
            return Err(JdkError::DownloadError(format!(
                "Downloaded archive failed size or SHA-256 verification: {}", target_path.display()
            )));
        }
        if Self::verify_file(&target_path, expected_size, checksum)? {
            return Ok(target_path);
        }
        if let Err(error) = part_path.persist(&target_path) {
            if Self::verify_file(&target_path, expected_size, checksum)? {
                return Ok(target_path);
            }
            return Err(JdkError::DownloadError(format!("Cannot save verified archive: {error}")));
        }
        Ok(target_path)
    }

    fn verify_file(path: &Path, expected_size: u64, checksum: Option<&str>) -> Result<bool> {
        if !path.is_file() { return Ok(false); }
        if std::fs::metadata(path)?.len() != expected_size { return Ok(false); }
        let Some(checksum) = checksum else { return Ok(true); };
        let mut file = std::fs::File::open(path)?;
        let mut buffer = [0_u8; 64 * 1024];
        let mut hasher = Sha256::new();
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 { break; }
            hasher.update(&buffer[..count]);
        }
        Ok(format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(checksum))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_requires_matching_size_and_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("jdk.zip");
        std::fs::write(&archive, b"abc").unwrap();
        let checksum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(Downloader::verify_file(&archive, 3, Some(checksum)).unwrap());
        assert!(!Downloader::verify_file(&archive, 4, Some(checksum)).unwrap());
        assert!(!Downloader::verify_file(&archive, 3, Some(&"0".repeat(64))).unwrap());
    }

    #[test]
    fn configured_download_directory_resolves_relative_to_config() {
        let base = Path::new("C:/jsh");
        assert_eq!(Downloader::resolve_download_dir(base, Path::new("cache")).unwrap(), base.join("cache"));
        assert!(Downloader::resolve_download_dir(base, Path::new("")).is_err());
    }
}
