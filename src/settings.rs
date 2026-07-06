use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub enable_transparent_mode: bool,
    pub enable_hover_show_hide: bool,
    pub visible_opacity_percent: u8,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            enable_transparent_mode: false,
            enable_hover_show_hide: false,
            visible_opacity_percent: 85,
        }
    }
}

impl UiSettings {
    pub fn load() -> Self {
        let Ok(path) = settings_path() else {
            return Self::default();
        };
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut settings = serde_json::from_str::<Self>(&text).unwrap_or_default();
        settings.normalize();
        settings
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.normalize();
        let path = settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temporary, path)?;
        Ok(())
    }

    pub fn visible_alpha(&self) -> u8 {
        ((u16::from(self.visible_opacity_percent) * 255) / 100) as u8
    }

    pub fn normalize(&mut self) {
        self.visible_opacity_percent = self.visible_opacity_percent.clamp(20, 100);
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe().context("failed to locate executable")?;
    Ok(executable
        .parent()
        .context("executable has no parent directory")?
        .join("data")
        .join("settings.json"))
}
