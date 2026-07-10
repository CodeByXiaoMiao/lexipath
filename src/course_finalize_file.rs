use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::course::CoursePack;
use crate::course_finalize::finalize_course;

pub fn run(arguments: &[String]) -> anyhow::Result<()> {
    let mut input = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .with_context(|| format!("missing value after {}", arguments[index]))?;
        match arguments[index].as_str() {
            "--input" => input = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            other => bail!("unknown course finalization option: {other}"),
        }
        index += 2;
    }

    let input = input.context("--input is required")?;
    let output = output.context("--output is required")?;
    let bytes = std::fs::read(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let mut course: CoursePack = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse {}", input.display()))?;
    finalize_course(&mut course)?;
    std::fs::write(&output, serde_json::to_vec(&course)?)?;
    println!("finalized course written to {}", output.display());
    Ok(())
}
