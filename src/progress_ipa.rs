use crate::progress_store::ProgressStore;
use crate::scheduler::current_day;

impl ProgressStore {
    pub fn ipa_completed_days(&self) -> usize {
        self.data.ipa_completed_days
    }

    pub fn ipa_completed_today(&self) -> bool {
        self.data.ipa_last_completed_day == Some(current_day())
    }

    pub fn set_ipa_current_day_number(
        &mut self,
        day_number: usize,
        total_days: usize,
    ) -> anyhow::Result<()> {
        let target = day_number.clamp(1, total_days.max(1));
        self.data.ipa_completed_days = target.saturating_sub(1);
        self.data.ipa_last_completed_day = None;
        self.save_exact()
    }

    pub fn complete_ipa_day(&mut self, total_days: usize) -> anyhow::Result<()> {
        self.data.ipa_completed_days = self
            .data
            .ipa_completed_days
            .saturating_add(1)
            .min(total_days);
        self.data.ipa_last_completed_day = Some(current_day());
        self.save()
    }
}
