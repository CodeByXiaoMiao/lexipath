use crate::progress_store::ProgressStore;

impl ProgressStore {
    pub fn complete_review(&mut self, review_id: u64) -> anyhow::Result<()> {
        if let Some(review) = self.data.reviews.iter_mut().find(|item| item.id == review_id) {
            review.completed = true;
        }
        self.save()
    }
}
