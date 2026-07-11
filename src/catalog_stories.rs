use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::controlled_english::{infer_morph_class, sequence_count, tokenize};
use crate::course::{CoursePack, Lesson};

const ALLOWED_CHARACTER_NAMES: &[&str] = &[
    "alex", "anna", "ben", "emma", "jack", "leo", "lily", "lucy", "mia", "nina",
    "sam", "tom",
];

#[derive(Debug, Clone, Deserialize)]
struct StoryAsset {
    lesson_id: String,
    title: String,
    level: String,
    characters: Vec<String>,
    sentences: Vec<String>,
    translations: Vec<String>,
    arc: StoryArc,
}

#[derive(Debug, Clone, Deserialize)]
struct StoryArc {
    setup_sentence: usize,
    goal_sentence: usize,
    problem_sentence: usize,
    attempt_sentences: Vec<usize>,
    turn_sentence: usize,
    #[serde(default)]
    reveal_sentence: Option<usize>,
    resolution_sentence: usize,
}

static STORIES: OnceLock<HashMap<String, StoryAsset>> = OnceLock::new();

pub fn apply_curated_story(lesson: &mut Lesson) -> bool {
    let Some(story) = stories().get(&lesson.id) else {
        return false;
    };
    lesson.reading.title = story.title.clone();
    lesson.reading.sentences = story.sentences.clone();
    true
}

pub fn has_curated_story(lesson_id: &str) -> bool {
    stories().contains_key(lesson_id)
}

pub fn curated_translation(lesson_id: &str, english: &str) -> Option<&'static str> {
    let story = stories().get(lesson_id)?;
    let index = story
        .sentences
        .iter()
        .position(|sentence| sentence == english)?;
    story.translations.get(index).map(String::as_str)
}

pub fn missing_story_ids(course: &CoursePack) -> Vec<String> {
    course
        .stages
        .iter()
        .filter(|stage| stage.id != "foundation-words")
        .flat_map(|stage| stage.lessons.iter())
        .filter(|lesson| !lesson.is_stage_assessment())
        .filter(|lesson| !has_curated_story(&lesson.id))
        .map(|lesson| lesson.id.clone())
        .collect()
}

pub fn validate_story_bank_coverage(course: &CoursePack) -> anyhow::Result<()> {
    let missing = missing_story_ids(course);
    if missing.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "LLM reading bank is incomplete: {} lessons are missing; first items: {}",
            missing.len(),
            missing
                .into_iter()
                .take(20)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub fn validate_curated_story(lesson: &Lesson) -> Vec<String> {
    let Some(story) = stories().get(&lesson.id) else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    let field = format!("lesson {} / curated_story", lesson.id);

    if lesson.reading.title != story.title || lesson.reading.sentences != story.sentences {
        issues.push(format!(
            "{field}: finalized reading does not match its curated asset"
        ));
    }

    validate_characters(story, &field, &mut issues);
    validate_arc(story, &field, &mut issues);
    validate_shape(story, &field, &mut issues);

    let reading_tokens = tokenize(&lesson.full_reading_text());
    for word in &lesson.new_words {
        let entry_tokens = tokenize(&word.text);
        let class = infer_morph_class(&word.text, &word.meaning, &word.phrase, &word.example);
        let count = sequence_count(&reading_tokens, &entry_tokens, class);
        if count < 2 {
            issues.push(format!(
                "{field}: target '{}' must appear at least twice including controlled inflections; found {count}",
                word.text
            ));
        }
        let separate_sentences = story
            .sentences
            .iter()
            .filter(|sentence| sequence_count(&tokenize(sentence), &entry_tokens, class) > 0)
            .count();
        if separate_sentences < 2 {
            issues.push(format!(
                "{field}: target '{}' must appear in at least two separate story sentences",
                word.text
            ));
        }
    }

    issues
}

pub fn character_name_is_allowed(name: &str) -> bool {
    ALLOWED_CHARACTER_NAMES.contains(&name.to_ascii_lowercase().as_str())
}

pub fn allowed_character_tokens(lesson_id: &str) -> HashSet<String> {
    stories()
        .get(lesson_id)
        .into_iter()
        .flat_map(|story| story.characters.iter())
        .flat_map(|name| tokenize(name))
        .collect()
}

fn stories() -> &'static HashMap<String, StoryAsset> {
    STORIES.get_or_init(|| {
        let records: Vec<StoryAsset> = serde_json::from_str(include_str!(
            "../assets/course-stories/curated.json"
        ))
        .expect("curated story data is invalid");
        let mut output = HashMap::new();
        for record in records {
            let key = record.lesson_id.clone();
            assert!(
                output.insert(key.clone(), record).is_none(),
                "duplicate curated story: {key}"
            );
        }
        output
    })
}

fn validate_characters(story: &StoryAsset, field: &str, issues: &mut Vec<String>) {
    if story.characters.is_empty() || story.characters.len() > 4 {
        issues.push(format!(
            "{field}.characters: expected one to four named characters"
        ));
    }
    let mut seen = HashSet::new();
    for name in &story.characters {
        let lower = name.to_ascii_lowercase();
        if !ALLOWED_CHARACTER_NAMES.contains(&lower.as_str()) {
            issues.push(format!(
                "{field}.characters: unsupported character name '{name}'"
            ));
        }
        if !seen.insert(lower.clone()) {
            issues.push(format!(
                "{field}.characters: duplicate character name '{name}'"
            ));
        }
        let appearances = story
            .sentences
            .iter()
            .flat_map(|sentence| tokenize(sentence))
            .filter(|token| token == &lower)
            .count();
        if appearances < 2 {
            issues.push(format!(
                "{field}.characters: '{name}' must appear at least twice; found {appearances}"
            ));
        }
    }
}

fn validate_arc(story: &StoryAsset, field: &str, issues: &mut Vec<String>) {
    let len = story.sentences.len();
    let arc = &story.arc;
    let indexes = [
        arc.setup_sentence,
        arc.goal_sentence,
        arc.problem_sentence,
        arc.turn_sentence,
        arc.resolution_sentence,
    ];
    if indexes.iter().any(|index| *index >= len)
        || arc.attempt_sentences.iter().any(|index| *index >= len)
        || arc.reveal_sentence.is_some_and(|index| index >= len)
    {
        issues.push(format!(
            "{field}.arc: sentence index is outside the story"
        ));
        return;
    }
    if arc.attempt_sentences.len() < 2 {
        issues.push(format!(
            "{field}.arc: at least two attempts are required"
        ));
    }
    if !(arc.setup_sentence <= arc.goal_sentence
        && arc.goal_sentence < arc.problem_sentence
        && arc.problem_sentence < arc.turn_sentence
        && arc.turn_sentence < arc.resolution_sentence)
    {
        issues.push(format!(
            "{field}.arc: narrative events are not in chronological order"
        ));
    }
    if arc
        .attempt_sentences
        .iter()
        .any(|index| *index <= arc.problem_sentence || *index >= arc.turn_sentence)
    {
        issues.push(format!(
            "{field}.arc: attempts must occur after the problem and before the turn"
        ));
    }
    if let Some(reveal) = arc.reveal_sentence {
        if reveal <= arc.turn_sentence || reveal >= arc.resolution_sentence {
            issues.push(format!(
                "{field}.arc: reveal must occur after the turn and before the resolution"
            ));
        }
    }
}

fn validate_shape(story: &StoryAsset, field: &str, issues: &mut Vec<String>) {
    let (min_sentences, max_sentences, min_connectors) = match story.level.as_str() {
        "A1" => (10, 16, 4),
        "A2" => (12, 18, 5),
        "B1" => (14, 22, 6),
        "B2" => (16, 26, 7),
        _ => {
            issues.push(format!(
                "{field}.level: unsupported level '{}'",
                story.level
            ));
            (10, 26, 4)
        }
    };
    if !(min_sentences..=max_sentences).contains(&story.sentences.len()) {
        issues.push(format!(
            "{field}.sentences: {} story requires {min_sentences} to {max_sentences} sentences; found {}",
            story.level,
            story.sentences.len()
        ));
    }
    if story.translations.len() != story.sentences.len() {
        issues.push(format!(
            "{field}.translations: expected one Chinese translation per sentence; found {} translations for {} sentences",
            story.translations.len(),
            story.sentences.len()
        ));
    }
    for (index, translation) in story.translations.iter().enumerate() {
        let has_chinese = translation
            .chars()
            .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character));
        if translation.trim().is_empty() || !has_chinese {
            issues.push(format!(
                "{field}.translations[{index}]: expected a non-empty Chinese translation"
            ));
        }
    }

    let mut normalized = HashSet::new();
    for sentence in &story.sentences {
        if !normalized.insert(sentence.trim().to_ascii_lowercase()) {
            issues.push(format!("{field}.sentences: duplicate sentence found"));
        }
    }

    let signatures = story
        .sentences
        .iter()
        .map(|sentence| tokenize(sentence).into_iter().take(2).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if signatures
        .windows(3)
        .any(|window| window[0] == window[1] && window[1] == window[2])
    {
        issues.push(format!(
            "{field}.sentences: three consecutive sentences use the same opening frame"
        ));
    }

    let connectors = [
        "after", "although", "because", "before", "but", "however", "if", "so", "then",
        "though", "when", "while",
    ];
    let story_tokens = story
        .sentences
        .iter()
        .flat_map(|sentence| tokenize(sentence))
        .collect::<HashSet<_>>();
    let connector_count = connectors
        .iter()
        .filter(|connector| story_tokens.contains(**connector))
        .count();
    if connector_count < min_connectors {
        issues.push(format!(
            "{field}.sentences: expected at least {min_connectors} different narrative connectors; found {connector_count}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_story_registry_loads_the_complex_trip_story() {
        let story = stories().get("a1-unit-047").expect("trip story");
        assert_eq!(story.level, "A1");
        assert_eq!(
            story.characters,
            vec!["Tom".to_owned(), "Anna".to_owned()]
        );
        assert!(story.sentences.len() >= 10);
        assert_eq!(story.sentences.len(), story.translations.len());
        assert_eq!(
            curated_translation(
                "a1-unit-047",
                "Tom is a young writer, and Anna is an adult."
            ),
            Some("汤姆是一名年轻作家，安娜是一位成年人。")
        );
    }
}
