use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::course::CoursePack;
use crate::course_finalize::finalize_course_with_options;

pub fn run(arguments: &[String]) -> anyhow::Result<()> {
    let mut input = None;
    let mut output = None;
    let mut require_llm_readings = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--require-llm-readings" => {
                require_llm_readings = true;
                index += 1;
            }
            "--input" | "--output" => {
                let value = arguments
                    .get(index + 1)
                    .with_context(|| format!("missing value after {}", arguments[index]))?;
                if arguments[index] == "--input" {
                    input = Some(PathBuf::from(value));
                } else {
                    output = Some(PathBuf::from(value));
                }
                index += 2;
            }
            other => bail!("unknown course finalization option: {other}"),
        }
    }

    let input = input.context("--input is required")?;
    let output = output.context("--output is required")?;
    let bytes = std::fs::read(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let mut course: CoursePack = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse {}", input.display()))?;
    finalize_course_with_options(&mut course, require_llm_readings)?;
    std::fs::write(&output, serde_json::to_vec(&course)?)?;
    println!("finalized course written to {}", output.display());
    Ok(())
}
