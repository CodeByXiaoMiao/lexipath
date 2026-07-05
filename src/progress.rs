use crate::scheduler::current_day;

#[derive(Debug, Clone)]
pub struct ReviewRecord {
    pub id: i64,
    pub lesson_id: String,
    pub step: usize,
}

pub fn today() -> i64 {
    current_day()
}
