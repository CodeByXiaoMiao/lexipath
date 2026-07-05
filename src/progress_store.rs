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
        let temporary = self.path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(&self.data)?)?;
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
