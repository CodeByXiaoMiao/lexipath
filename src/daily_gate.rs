use crate::course::STAGE_ASSESSMENT_PREFIX;
use crate::progress_store::ProgressStore;
use crate::scheduler::current_day;

pub const NEW_UNITS_PER_DAY: usize = 2;

impl ProgressStore {
    pub fn new_units_completed_today(&self) -> usize {
        let today = current_day();
        self.data
            .lessons
            .iter()
            .filter(|lesson| {
                lesson.completed_day == today
                    && !lesson.lesson_id.starts_with(STAGE_ASSESSMENT_PREFIX)
            })
            .count()
    }

    pub fn vocabulary_locked_today(&self) -> bool {
        let assessment_is_current = self
            .data
            .current_lesson_id
            .as_deref()
            .map(|lesson_id| lesson_id.starts_with(STAGE_ASSESSMENT_PREFIX))
            .unwrap_or(false);
        !assessment_is_current
            && self.due_count() == 0
            && self.new_units_completed_today() >= NEW_UNITS_PER_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_vocabulary_limit_is_twelve_words() {
        assert_eq!(NEW_UNITS_PER_DAY, 2);
        assert_eq!(NEW_UNITS_PER_DAY * 6, 12);
    }
}
