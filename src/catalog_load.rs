use std::fs;

use anyhow::Context;

use crate::catalog_daily::add_daily_combined_readings;
use crate::course::CoursePack;
use crate::embedded_course;

pub fn load() -> anyhow::Result<CoursePack> {
    let executable = std::env::current_exe().context("failed to locate executable")?;
    let external_path = executable
        .parent()
        .context("executable has no parent directory")?
        .join("course.json");

    let mut course = if external_path.exists() {
        let data = fs::read(&external_path)
            .with_context(|| format!("failed to read {}", external_path.display()))?;
        serde_json::from_slice(&data)
            .with_context(|| format!("failed to parse {}", external_path.display()))?
    } else {
        embedded_course::load()?
    };

    for stage in &mut course.stages {
        if stage.id.starts_with("oxford-") {
            add_daily_combined_readings(stage);
        }
    }

    Ok(course)
}
