use std::fs;
use std::path::PathBuf;

use anyhow::Context;

use crate::progress_data::ProgressData;

pub struct ProgressStore {
    path: PathBuf,
    pub data: ProgressData,
}

impl ProgressStore {
    pub fn open() -> anyhow::Result<Self> {
        let path = progress_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = if path.exists() {
            let text = fs::read_to_string(&path)?;
            serde_json::from_str(&text).context("failed to read progress data")?
        } else {
            ProgressData::default()
        };
        Ok(Self { path, data })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut merged = self.data.clone();
        if self.path.exists() {
            if let Ok(text) = fs::read_to_string(&self.path) {
                if let Ok(on_disk) = serde_json::from_str::<ProgressData>(&text) {
                    merged.ipa_completed_days =
                        merged.ipa_completed_days.max(on_disk.ipa_completed_days);
                    merged.ipa_last_completed_day = match (
                        merged.ipa_last_completed_day,
                        on_disk.ipa_last_completed_day,
                    ) {
                        (Some(memory), Some(disk)) => Some(memory.max(disk)),
                        (Some(day), None) | (None, Some(day)) => Some(day),
                        (None, None) => None,
                    };
                }
            }
        }

        let temporary = self.path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(&merged)?)?;
        fs::rename(temporary, &self.path)?;
        Ok(())
    }
}

fn progress_path() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe().context("failed to locate executable")?;
    Ok(executable
        .parent()
        .context("executable has no parent directory")?
        .join("data")
        .join("progress.json"))
}
