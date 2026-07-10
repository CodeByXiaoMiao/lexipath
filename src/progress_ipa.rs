use crate::progress_store::ProgressStore;
use crate::scheduler::current_day;

impl ProgressStore {
    pub fn ipa_completed_days(&self) -> usize {
        self.data.ipa_completed_days
    }

    pub fn ipa_current_day_number(&self, total_days: usize) -> usize {
        self.data
            .ipa_active_day_number
            .unwrap_or_else(|| self.data.ipa_completed_days.saturating_add(1))
            .clamp(1, total_days.max(1))
    }

    pub fn ipa_completed_today(&self) -> bool {
        self.data.ipa_last_completed_day == Some(current_day())
    }

    pub fn set_ipa_current_day_number(
        &mut self,
        day_number: usize,
        total_days: usize,
    ) -> anyhow::Result<()> {
        self.data.ipa_active_day_number = Some(day_number.clamp(1, total_days.max(1)));
        self.save_exact()
    }

    pub fn complete_ipa_day(
        &mut self,
        day_number: usize,
        total_days: usize,
    ) -> anyhow::Result<()> {
        let completed = day_number.clamp(1, total_days.max(1));
        self.data.ipa_completed_days = self.data.ipa_completed_days.max(completed).min(total_days);
        self.data.ipa_active_day_number = if completed < total_days {
            Some(completed + 1)
        } else {
            None
        };
        self.data.ipa_last_completed_day = Some(current_day());
        self.save()
    }
}
