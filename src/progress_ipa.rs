use crate::progress_store::ProgressStore;

impl ProgressStore {
    pub fn ipa_completed_days(&self) -> usize {
        self.data.ipa_completed_days
    }

    pub fn complete_ipa_day(&mut self, total_days: usize) -> anyhow::Result<()> {
        self.data.ipa_completed_days = self
            .data
            .ipa_completed_days
            .saturating_add(1)
            .min(total_days);
        self.save()
    }
}
