use anyhow::Context;

use crate::course::{CoursePack, Lesson};

const EXTRA_LESSONS: &[&str] = &[
    include_str!("../assets/courses/foundation-002.json"),
    include_str!("../assets/courses/foundation-003.json"),
    include_str!("../assets/courses/foundation-004.json"),
    include_str!("../assets/courses/foundation-005.json"),
    include_str!("../assets/courses/foundation-006.json"),
    include_str!("../assets/courses/foundation-007.json"),
    include_str!("../assets/courses/foundation-008.json"),
    include_str!("../assets/courses/foundation-008b.json"),
    include_str!("../assets/courses/foundation-009.json"),
    include_str!("../assets/courses/foundation-010.json"),
    include_str!("../assets/courses/foundation-011.json"),
    include_str!("../assets/courses/foundation-012.json"),
];

pub fn load() -> anyhow::Result<CoursePack> {
    let mut course = CoursePack::embedded()?;
    let stage = course
        .stages
        .first_mut()
        .context("foundation course has no stage")?;
    for source in EXTRA_LESSONS {
        let lesson: Lesson = serde_json::from_str(source)
            .context("failed to parse an embedded foundation lesson")?;
        stage.lessons.push(lesson);
    }
    course.version = 4;
    Ok(course)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::validate_course;

    #[test]
    fn expanded_course_contains_only_opened_words() {
        let course = load().expect("course should load");
        assert_eq!(course.stages[0].lessons.len(), 13);
        assert_eq!(validate_course(&course), Ok(()));
    }
}
