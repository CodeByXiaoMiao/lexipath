use crate::course::Lesson;
use crate::engine::LearningSession;

pub fn due_practice_session(lesson: Lesson) -> LearningSession {
    let word_count = lesson.new_words.len();
    let mut session = LearningSession::new(lesson);
    for _ in 0..word_count {
        session.mark_word_audio_played();
        session.advance_word();
    }
    session
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::CoursePack;
    use crate::engine::Phase;

    #[test]
    fn due_practice_skips_first_exposure() {
        let course = CoursePack::embedded().expect("course");
        let session = due_practice_session(course.first_lesson().expect("lesson").clone());
        assert_eq!(session.phase(), Phase::Recognition);
    }
}
