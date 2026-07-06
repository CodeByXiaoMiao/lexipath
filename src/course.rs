use anyhow::Context;
use serde::{Deserialize, Serialize};

pub const STAGE_ASSESSMENT_PREFIX: &str = "stage-final-";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoursePack {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stage {
    pub id: String,
    pub title: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub new_words: Vec<WordItem>,
    pub sentences: Vec<SentenceItem>,
    pub reading: Reading,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordItem {
    pub id: String,
    pub text: String,
    pub ipa: String,
    pub meaning: String,
    pub phrase: String,
    pub example: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SentenceItem {
    pub text: String,
    pub meaning: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reading {
    pub title: String,
    pub sentences: Vec<String>,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Question {
    pub prompt: String,
    pub options: Vec<String>,
    pub correct_index: usize,
}

impl CoursePack {
    pub fn embedded() -> anyhow::Result<Self> {
        serde_json::from_str(include_str!("../assets/courses/foundation.json"))
            .context("failed to parse embedded foundation course")
    }

    pub fn first_lesson(&self) -> Option<&Lesson> {
        self.stages.first()?.lessons.first()
    }
}

impl Lesson {
    pub fn full_reading_text(&self) -> String {
        self.reading.sentences.join(" ")
    }

    pub fn is_stage_assessment(&self) -> bool {
        self.id.starts_with(STAGE_ASSESSMENT_PREFIX)
    }
}
