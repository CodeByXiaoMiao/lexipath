use crate::course::{CoursePack, Lesson};

pub trait CourseCatalog {
    fn lesson_count(&self) -> usize;
    fn lesson_by_id(&self, lesson_id: &str) -> Option<&Lesson>;
    fn next_lesson(&self, lesson_id: &str) -> Option<&Lesson>;
}

impl CourseCatalog for CoursePack {
    fn lesson_count(&self) -> usize {
        self.stages.iter().map(|stage| stage.lessons.len()).sum()
    }

    fn lesson_by_id(&self, lesson_id: &str) -> Option<&Lesson> {
        self.stages
            .iter()
            .flat_map(|stage| stage.lessons.iter())
            .find(|lesson| lesson.id == lesson_id)
    }

    fn next_lesson(&self, lesson_id: &str) -> Option<&Lesson> {
        let mut found = false;
        for lesson in self.stages.iter().flat_map(|stage| stage.lessons.iter()) {
            if found {
                return Some(lesson);
            }
            found = lesson.id == lesson_id;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_course_can_find_a_lesson() {
        let course = CoursePack::embedded().expect("course");
        let id = course.first_lesson().expect("lesson").id.clone();
        assert!(course.lesson_by_id(&id).is_some());
    }
}
