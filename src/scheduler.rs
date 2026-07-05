use std::time::{SystemTime, UNIX_EPOCH};

pub const REVIEW_OFFSETS_DAYS: [i64; 6] = [1, 3, 7, 14, 30, 60];

pub fn current_day() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or_default()
}

pub fn due_day(completed_day: i64, review_step: usize) -> Option<i64> {
    REVIEW_OFFSETS_DAYS
        .get(review_step)
        .map(|offset| completed_day + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_offsets_are_fixed() {
        assert_eq!(due_day(100, 0), Some(101));
        assert_eq!(due_day(100, 5), Some(160));
        assert_eq!(due_day(100, 6), None);
    }
}
