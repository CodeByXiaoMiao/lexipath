use std::collections::HashSet;

use serde::Deserialize;

use crate::course::{CoursePack, Lesson, Question, Reading, Stage};
use crate::validator::tokenize;

const TARGET_READING_WORDS: usize = 1_000;
const MIN_READING_WORDS: usize = 800;
const MAX_READING_WORDS: usize = 1_150;
const TARGET_QUESTIONS: usize = 20;
const MIN_QUESTIONS: usize = 15;

#[derive(Debug, Clone, Deserialize)]
struct AssessmentAsset {
    title: String,
    sentences: Vec<String>,
    questions: Vec<Question>,
}

pub fn append_required_stage_assessments(course: &mut CoursePack) {
    let mut learned_tokens = HashSet::<String>::new();

    for stage in &mut course.stages {
        for lesson in &stage.lessons {
            for word in &lesson.new_words {
                learned_tokens.extend(tokenize(&word.text));
            }
        }

        if !requires_final_assessment(&stage.id) {
            continue;
        }

        stage
            .lessons
            .retain(|lesson| !lesson.is_stage_assessment());
        if let Some(assessment) = build_stage_assessment(stage, &learned_tokens) {
            stage.lessons.push(assessment);
        }
    }
}

fn requires_final_assessment(stage_id: &str) -> bool {
    stage_id.starts_with("ogden-")
}

fn build_stage_assessment(
    stage: &Stage,
    learned_tokens: &HashSet<String>,
) -> Option<Lesson> {
    if let Some(asset) = curated_asset(&stage.id) {
        if asset_is_valid(&asset, learned_tokens) {
            return Some(Lesson {
                id: format!("stage-final-{}", stage.id),
                title: format!("{} · 阶段总结阅读", stage.title),
                new_words: Vec::new(),
                sentences: Vec::new(),
                reading: Reading {
                    title: asset.title,
                    sentences: asset.sentences,
                    questions: asset.questions,
                },
            });
        }
    }

    build_fallback_assessment(stage)
}

fn curated_asset(stage_id: &str) -> Option<AssessmentAsset> {
    let source = match stage_id {
        "ogden-850" => include_str!("../assets/stage-assessments/ogden-850.json"),
        _ => return None,
    };
    serde_json::from_str(source).ok()
}

fn asset_is_valid(asset: &AssessmentAsset, learned_tokens: &HashSet<String>) -> bool {
    let reading_words = asset
        .sentences
        .iter()
        .map(|sentence| tokenize(sentence).len())
        .sum::<usize>();
    if !(MIN_READING_WORDS..=MAX_READING_WORDS).contains(&reading_words)
        || !(MIN_QUESTIONS..=25).contains(&asset.questions.len())
    {
        return false;
    }

    let all_text_is_learned = asset
        .sentences
        .iter()
        .chain(
            asset
                .questions
                .iter()
                .flat_map(|question| question.options.iter()),
        )
        .flat_map(|text| tokenize(text))
        .all(|token| learned_tokens.contains(&token));
    if !all_text_is_learned {
        return false;
    }

    asset.questions.iter().all(|question| {
        question.options.len() >= 2
            && question.correct_index < question.options.len()
            && question.options.iter().collect::<HashSet<_>>().len()
                == question.options.len()
    })
}

fn build_fallback_assessment(stage: &Stage) -> Option<Lesson> {
    let source_lessons = stage
        .lessons
        .iter()
        .filter(|lesson| !lesson.is_stage_assessment())
        .collect::<Vec<_>>();
    if source_lessons.is_empty() {
        return None;
    }

    let sentences = collect_spread_sentences(&source_lessons);
    let questions = collect_spread_questions(&source_lessons);
    let word_count = sentences
        .iter()
        .map(|sentence| tokenize(sentence).len())
        .sum::<usize>();

    if word_count < MIN_READING_WORDS || questions.len() < MIN_QUESTIONS {
        return None;
    }

    Some(Lesson {
        id: format!("stage-final-{}", stage.id),
        title: format!("{} · 阶段总结阅读", stage.title),
        new_words: Vec::new(),
        sentences: Vec::new(),
        reading: Reading {
            title: "阶段总结长篇阅读".to_owned(),
            sentences,
            questions,
        },
    })
}

fn collect_spread_sentences(source_lessons: &[&Lesson]) -> Vec<String> {
    let max_sentence_count = source_lessons
        .iter()
        .map(|lesson| lesson.reading.sentences.len())
        .max()
        .unwrap_or_default();
    let mut seen = HashSet::<String>::new();
    let mut selected = Vec::<String>::new();
    let mut selected_words = 0usize;

    'spread: for sentence_index in 0..max_sentence_count {
        for lesson in source_lessons {
            let Some(sentence) = lesson.reading.sentences.get(sentence_index) else {
                continue;
            };
            let normalized = sentence.trim().to_ascii_lowercase();
            if normalized.is_empty() || !seen.insert(normalized) {
                continue;
            }

            let sentence_words = tokenize(sentence).len();
            if sentence_words == 0 {
                continue;
            }
            if selected_words >= MIN_READING_WORDS
                && selected_words + sentence_words > MAX_READING_WORDS
            {
                break 'spread;
            }

            selected.push(sentence.clone());
            selected_words += sentence_words;
            if selected_words >= TARGET_READING_WORDS {
                break 'spread;
            }
        }
    }

    if selected_words < MIN_READING_WORDS {
        'fill: loop {
            let before = selected_words;
            for lesson in source_lessons {
                for sentence in &lesson.reading.sentences {
                    let sentence_words = tokenize(sentence).len();
                    if sentence_words == 0 {
                        continue;
                    }
                    if selected_words + sentence_words > MAX_READING_WORDS {
                        break 'fill;
                    }
                    selected.push(sentence.clone());
                    selected_words += sentence_words;
                    if selected_words >= MIN_READING_WORDS {
                        break 'fill;
                    }
                }
            }
            if selected_words == before {
                break;
            }
        }
    }

    selected
}

fn collect_spread_questions(source_lessons: &[&Lesson]) -> Vec<Question> {
    let mut seen = HashSet::<String>::new();
    let mut candidates = Vec::<Question>::new();

    for lesson in source_lessons {
        for question in &lesson.reading.questions {
            if question.options.len() < 2 || question.correct_index >= question.options.len() {
                continue;
            }
            let unique_options = question.options.iter().collect::<HashSet<_>>();
            if unique_options.len() != question.options.len() {
                continue;
            }
            let key = format!("{}\u{1f}{}", question.prompt, question.options.join("\u{1e}"));
            if seen.insert(key) {
                candidates.push(question.clone());
            }
        }
    }

    if candidates.len() <= TARGET_QUESTIONS {
        return candidates;
    }

    (0..TARGET_QUESTIONS)
        .map(|index| {
            let source_index = index * candidates.len() / TARGET_QUESTIONS;
            candidates[source_index].clone()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{SentenceItem, WordItem};

    fn lesson(index: usize) -> Lesson {
        let sentences = (0..12)
            .map(|sentence| format!("I see book {index} {sentence}."))
            .collect::<Vec<_>>();
        Lesson {
            id: format!("unit-{index}"),
            title: format!("Unit {index}"),
            new_words: vec![WordItem {
                id: format!("word-{index}"),
                text: "book".to_owned(),
                ipa: "/bʊk/".to_owned(),
                meaning: "书".to_owned(),
                phrase: "a book".to_owned(),
                example: "I see a book.".to_owned(),
            }],
            sentences: vec![SentenceItem {
                text: "I see a book.".to_owned(),
                meaning: "我看见一本书。".to_owned(),
            }],
            reading: Reading {
                title: "阅读".to_owned(),
                sentences,
                questions: vec![Question {
                    prompt: format!("问题 {index}"),
                    options: vec!["I see a book.".to_owned(), "You see a book.".to_owned()],
                    correct_index: 0,
                }],
            },
        }
    }

    #[test]
    fn ogden_assessment_is_appended_once() {
        let mut course = CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "ogden-850".to_owned(),
                title: "Ogden".to_owned(),
                lessons: (0..100).map(lesson).collect(),
            }],
        };

        append_required_stage_assessments(&mut course);
        append_required_stage_assessments(&mut course);

        let assessments = course.stages[0]
            .lessons
            .iter()
            .filter(|lesson| lesson.is_stage_assessment())
            .count();
        assert_eq!(assessments, 1);
        let assessment = course.stages[0].lessons.last().expect("assessment");
        assert!(assessment.reading.questions.len() >= MIN_QUESTIONS);
        assert!(tokenize(&assessment.full_reading_text()).len() >= MIN_READING_WORDS);
    }

    #[test]
    fn curated_asset_rejects_unlearned_words() {
        let asset = AssessmentAsset {
            title: "测试".to_owned(),
            sentences: vec!["Unknown word.".to_owned(); 500],
            questions: vec![Question {
                prompt: "问题".to_owned(),
                options: vec!["Unknown word.".to_owned(), "Known word.".to_owned()],
                correct_index: 0,
            }; 20],
        };
        let learned = HashSet::from(["known".to_owned(), "word".to_owned()]);
        assert!(!asset_is_valid(&asset, &learned));
    }
}
