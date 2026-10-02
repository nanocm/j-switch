#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(target_os = "windows"))]
pub mod unix;

use crate::error::Result;
use std::path::Path;

pub trait EnvUpdater {
    fn update_java_home(&self, path: &Path) -> Result<()>;
}

#[cfg(target_os = "windows")]
pub type PlatformEnvUpdater = windows::WindowsEnvUpdater;

#[cfg(not(target_os = "windows"))]
pub type PlatformEnvUpdater = unix::UnixEnvUpdater;

pub fn get_env_updater() -> PlatformEnvUpdater {
    PlatformEnvUpdater::new()
}
