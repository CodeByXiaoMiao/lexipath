use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub id: u64,
    pub lesson_id: String,
    pub step: usize,
    pub due_day: i64,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonRecord {
    pub lesson_id: String,
    pub first_attempt_accuracy: f32,
    pub completed_day: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgressData {
    pub current_lesson_id: Option<String>,
    pub lessons: Vec<LessonRecord>,
    pub reviews: Vec<ReviewRecord>,
    pub next_review_id: u64,
}
