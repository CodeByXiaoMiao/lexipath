use crate::catalog_polish::polish_generated_content;
use crate::catalog_quality::validate_content_quality;
use crate::course::CoursePack;
use crate::validator::validate_course;

pub fn finalize_course(course: &mut CoursePack) -> anyhow::Result<()> {
    polish_generated_content(course);
    validate_course(course).map_err(|errors| {
        anyhow::anyhow!(
            "zero-unknown validation failed: {}",
            errors
                .into_iter()
                .take(80)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    })?;
    validate_content_quality(course).map_err(|issues| {
        anyhow::anyhow!(
            "content quality validation failed: {}",
            issues
                .into_iter()
                .take(120)
                .map(|issue| issue.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    })?;
    Ok(())
}
