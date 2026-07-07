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
    #[serde(default)]
    pub ipa_completed_days: usize,
    #[serde(default)]
    pub ipa_last_completed_day: Option<i64>,
    #[serde(default)]
    pub learning_day: Option<i64>,
    #[serde(default)]
    pub new_units_completed_today: usize,
    #[serde(default)]
    pub manual_new_units_override_day: Option<i64>,
    #[serde(default)]
    pub current_lesson_id: Option<String>,
    #[serde(default)]
    pub course_complete: bool,
    #[serde(default)]
    pub lessons: Vec<LessonRecord>,
    #[serde(default)]
    pub reviews: Vec<ReviewRecord>,
    #[serde(default)]
    pub next_review_id: u64,
}
