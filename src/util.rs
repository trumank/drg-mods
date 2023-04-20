use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

#[cfg(target_os = "windows")]
pub const TARGET: &str = "WindowsNoEditor";
#[cfg(target_os = "linux")]
pub const TARGET: &str = "LinuxNoEditor";

pub fn get_fsd_pak() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FSD_PAK") {
        Ok(PathBuf::from(path))
    } else if let Some(path) = steamlocate::SteamDir::locate().and_then(|mut steamdir| {
        steamdir
            .app(&548430)
            .map(|a| a.path.join("FSD/Content/Paks/FSD-WindowsNoEditor.pak"))
    }) {
        Ok(path)
    } else {
        bail!("Unable to locate FSD-WindowsNoEditor.pak. Specify it manually with the FSD_PAK env var")
    }
}

pub fn get_cooked_dir() -> PathBuf {
    Path::new("Saved/Cooked").join(TARGET)
}
