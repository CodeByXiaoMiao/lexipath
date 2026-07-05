use crate::progress_data::ReviewRecord;
use crate::progress_store::ProgressStore;
use crate::scheduler::current_day;

impl ProgressStore {
    pub fn current_lesson_id(&self) -> Option<&str> {
        self.data.current_lesson_id.as_deref()
    }

    pub fn completed_count(&self) -> usize {
        self.data.lessons.len()
    }

    pub fn due_count(&self) -> usize {
        let today = current_day();
        self.data
            .reviews
            .iter()
            .filter(|review| !review.completed && review.due_day <= today)
            .count()
    }

    pub fn next_due_review(&self) -> Option<&ReviewRecord> {
        let today = current_day();
        self.data
            .reviews
            .iter()
            .filter(|review| !review.completed && review.due_day <= today)
            .min_by_key(|review| (review.due_day, review.id))
    }
}
