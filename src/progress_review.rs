use crate::progress_store::ProgressStore;

use crate::progress_data::ProgressData;
use crate::scheduler::current_day;

impl ProgressStore {
    pub fn complete_review(
        &mut self,
        review_id: u64,
        first_attempt_accuracy: f32,
    ) -> anyhow::Result<()> {
        let completed = self
            .data
            .reviews
            .iter()
            .find(|item| item.id == review_id)
            .map(|item| (item.lesson_id.clone(), item.step));

        if let Some(review) = self
            .data
            .reviews
            .iter_mut()
            .find(|item| item.id == review_id)
        {
            review.completed = true;
        }

        if let Some((lesson_id, step)) = completed {
            bring_failed_review_forward(
                &mut self.data,
                &lesson_id,
                step,
                first_attempt_accuracy,
                current_day(),
            );
        }

        self.save()
    }
}

fn bring_failed_review_forward(
    data: &mut ProgressData,
    lesson_id: &str,
    completed_step: usize,
    first_attempt_accuracy: f32,
    today: i64,
) {
    if first_attempt_accuracy >= 0.8 {
        return;
    }

    let Some(next_review) = data
        .reviews
        .iter_mut()
        .filter(|item| item.lesson_id == lesson_id && !item.completed && item.step > completed_step)
        .min_by_key(|item| item.step)
    else {
        return;
    };

    next_review.due_day = next_review.due_day.min(today + 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress_data::ReviewRecord;

    fn reviews() -> ProgressData {
        ProgressData {
            reviews: vec![
                ReviewRecord {
                    id: 1,
                    lesson_id: "lesson".to_owned(),
                    step: 0,
                    due_day: 10,
                    completed: true,
                },
                ReviewRecord {
                    id: 2,
                    lesson_id: "lesson".to_owned(),
                    step: 1,
                    due_day: 20,
                    completed: false,
                },
                ReviewRecord {
                    id: 3,
                    lesson_id: "lesson".to_owned(),
                    step: 2,
                    due_day: 30,
                    completed: false,
                },
            ],
            ..ProgressData::default()
        }
    }

    #[test]
    fn failed_review_brings_only_the_next_review_forward() {
        let mut data = reviews();

        bring_failed_review_forward(&mut data, "lesson", 0, 0.5, 12);

        assert_eq!(data.reviews[1].due_day, 13);
        assert_eq!(data.reviews[2].due_day, 30);
    }

    #[test]
    fn successful_review_keeps_the_existing_schedule() {
        let mut data = reviews();

        bring_failed_review_forward(&mut data, "lesson", 0, 0.8, 12);

        assert_eq!(data.reviews[1].due_day, 20);
    }
}
