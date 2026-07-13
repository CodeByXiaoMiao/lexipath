#[path = "catalog_template_apply.rs"]
mod catalog_template_apply;

use crate::catalog_example_translations::validate_example_translation_bank;
use crate::catalog_formalize::{formalize_generated_lessons, validate_formalized_course};
use crate::catalog_polish::polish_generated_content;
use crate::catalog_quality::validate_content_quality;
use crate::catalog_reviewed_stage_apply::apply_reviewed_stage_templates;
use crate::catalog_stories::validate_story_bank_coverage;
use crate::course::CoursePack;
use crate::stage_assessment::append_required_stage_assessments;
use crate::validator::validate_course;

pub fn finalize_course(course: &mut CoursePack) -> anyhow::Result<()> {
    finalize_course_with_options(course, false)
}

pub fn finalize_course_with_options(
    course: &mut CoursePack,
    require_llm_readings: bool,
) -> anyhow::Result<()> {
    polish_generated_content(course);
    catalog_template_apply::apply_reviewed_templates(course);
    apply_reviewed_stage_templates(course);
    formalize_generated_lessons(course);
    append_required_stage_assessments(course);
    validate_finalized_course_with_options(course, require_llm_readings)
}

pub fn validate_finalized_course(course: &CoursePack) -> anyhow::Result<()> {
    validate_finalized_course_with_options(course, false)
}

fn validate_finalized_course_with_options(
    course: &CoursePack,
    require_llm_readings: bool,
) -> anyhow::Result<()> {
    if require_llm_readings {
        validate_story_bank_coverage(course)?;
    }

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
    validate_formalized_course(course)?;
    validate_example_translation_bank(course)?;
    Ok(())
}
