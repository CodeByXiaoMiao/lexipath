use crate::progress_data::{LessonRecord, ReviewRecord};
use crate::progress_store::ProgressStore;
use crate::scheduler::{current_day, due_day, REVIEW_OFFSETS_DAYS};

impl ProgressStore {
    pub fn set_current_lesson_id(&mut self, lesson_id: &str) -> anyhow::Result<()> {
        self.data.current_lesson_id = Some(lesson_id.to_owned());
        self.save()
    }

    pub fn complete_lesson(
        &mut self,
        lesson_id: &str,
        first_attempt_accuracy: f32,
    ) -> anyhow::Result<()> {
        let completed_day = current_day();
        self.data.lessons.retain(|item| item.lesson_id != lesson_id);
        self.data.lessons.push(LessonRecord {
            lesson_id: lesson_id.to_owned(),
            first_attempt_accuracy,
            completed_day,
        });

        for step in 0..REVIEW_OFFSETS_DAYS.len() {
            let exists = self
                .data
                .reviews
                .iter()
                .any(|item| item.lesson_id == lesson_id && item.step == step);
            if exists {
                continue;
            }
            let Some(target_day) = due_day(completed_day, step) else {
                continue;
            };
            self.data.next_review_id += 1;
            self.data.reviews.push(ReviewRecord {
                id: self.data.next_review_id,
                lesson_id: lesson_id.to_owned(),
                step,
                due_day: target_day,
                completed: false,
            });
        }
        self.save()
    }
}
