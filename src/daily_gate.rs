use crate::progress_store::ProgressStore;
use crate::scheduler::current_day;

pub const NEW_UNITS_PER_DAY: usize = 2;

impl ProgressStore {
    pub fn new_units_completed_today(&self) -> usize {
        let today = current_day();
        self.data
            .lessons
            .iter()
            .filter(|lesson| lesson.completed_day == today)
            .count()
    }

    pub fn vocabulary_locked_today(&self) -> bool {
        self.due_count() == 0 && self.new_units_completed_today() >= NEW_UNITS_PER_DAY
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
