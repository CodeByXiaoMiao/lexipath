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
    #[serde(default)]
    paragraphs: Vec<ParagraphRange>,
    arc: StoryArc,
}

#[derive(Debug, Clone, Deserialize)]
struct ParagraphRange {
    start: usize,
    end: usize,
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

pub fn curated_paragraph_ranges(lesson_id: &str, sentence_count: usize) -> Vec<(usize, usize)> {
    let Some(story) = stories().get(lesson_id) else {
        return fallback_paragraph_ranges(sentence_count);
    };
    if valid_paragraph_ranges(&story.paragraphs, sentence_count) {
        return story
            .paragraphs
            .iter()
            .map(|range| (range.start, range.end))
            .collect();
    }
    fallback_paragraph_ranges(sentence_count)
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

pub fn validate_curated_story(lesson: &Lesson, known_tokens: &HashSet<String>) -> Vec<String> {
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
    validate_shape(story, known_tokens, &field, &mut issues);
    validate_paragraphs(story, &field, &mut issues);
    validate_cohesion(
        story,
        &lesson
            .new_words
            .iter()
            .flat_map(|word| tokenize(&word.text))
            .collect::<HashSet<_>>(),
        &field,
        &mut issues,
    );

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

#[allow(dead_code)]
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

fn valid_paragraph_ranges(ranges: &[ParagraphRange], sentence_count: usize) -> bool {
    !ranges.is_empty()
        && ranges[0].start == 0
        && ranges.last().is_some_and(|range| range.end == sentence_count)
        && ranges.iter().all(|range| range.start < range.end)
        && ranges.windows(2).all(|window| window[0].end == window[1].start)
}

fn validate_paragraphs(story: &StoryAsset, field: &str, issues: &mut Vec<String>) {
    if story.paragraphs.is_empty() {
        return;
    }
    if !valid_paragraph_ranges(&story.paragraphs, story.sentences.len()) {
        issues.push(format!(
            "{field}.paragraphs: ranges must be contiguous and cover every sentence"
        ));
    }
}

fn fallback_paragraph_ranges(sentence_count: usize) -> Vec<(usize, usize)> {
    if sentence_count == 0 {
        return Vec::new();
    }
    let paragraph_count = match sentence_count {
        1..=6 => 1,
        7..=12 => 2,
        _ => 3,
    };
    let base = sentence_count / paragraph_count;
    let remainder = sentence_count % paragraph_count;
    let mut ranges = Vec::with_capacity(paragraph_count);
    let mut start = 0;
    for index in 0..paragraph_count {
        let size = base + usize::from(index < remainder);
        ranges.push((start, start + size));
        start += size;
    }
    ranges
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
        if appearances < 1 {
            issues.push(format!(
                "{field}.characters: '{name}' must appear at least once; found {appearances}"
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
        .any(|index| *index <= arc.goal_sentence || *index >= arc.turn_sentence)
    {
        issues.push(format!(
            "{field}.arc: attempts must occur after the goal and before the turn"
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

fn validate_shape(
    story: &StoryAsset,
    known_tokens: &HashSet<String>,
    field: &str,
    issues: &mut Vec<String>,
) {
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
    validate_story_language(story, field, issues);

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
    let available_connector_count = connectors
        .iter()
        .filter(|connector| known_tokens.contains(**connector))
        .count();
    // A reading article must carry the learner through a connected event;
    // the arc alone is not enough when every sentence is an isolated example.
    let required_connectors = min_connectors.min(available_connector_count);
    if connector_count < required_connectors {
        issues.push(format!(
            "{field}.sentences: expected at least {required_connectors} available narrative connectors; found {connector_count}"
        ));
    }
    let (min_words, max_words) = match story.level.as_str() {
        "A1" => (60, 90),
        "A2" => (90, 130),
        "B1" => (130, 180),
        "B2" => (180, 240),
        _ => (60, 240),
    };
    let word_count = story
        .sentences
        .iter()
        .map(|sentence| tokenize(sentence).len())
        .sum::<usize>();
    if !(min_words..=max_words).contains(&word_count) {
        issues.push(format!(
            "{field}.sentences: {} story requires {min_words}-{max_words} words; found {word_count}",
            story.level
        ));
    }
}

fn validate_story_language(story: &StoryAsset, field: &str, issues: &mut Vec<String>) {
    for (index, sentence) in story.sentences.iter().enumerate() {
        let lower = sentence.to_ascii_lowercase();
        for rejected in [
            "can be near",
            "has no humour about",
            "are with all the people in a room",
            "now has a black colour",
            "can not get it, but he can look at the box as anna can",
        ] {
            if lower.contains(rejected) {
                issues.push(format!(
                    "{field}.sentences[{index}]: rejected unnatural English frame '{rejected}'"
                ));
            }
        }
    }

    for (index, translation) in story.translations.iter().enumerate() {
        if translation
            .chars()
            .any(|character| character.is_ascii_alphabetic())
        {
            issues.push(format!(
                "{field}.translations[{index}]: translation contains an untranslated English fragment"
            ));
        }
        if !translation.ends_with(['。', '！', '？']) {
            issues.push(format!(
                "{field}.translations[{index}]: translation must end with Chinese sentence punctuation"
            ));
        }
        for rejected in [
            "这是一个书。",
            "这是一个食物。",
            "这是一十亿。",
            "这是几把剪刀。",
            "他是女性。",
            "机器翻译",
            "例句中",
            "（例句中文译文缺失）",
        ] {
            if translation.contains(rejected) {
                issues.push(format!(
                    "{field}.translations[{index}]: rejected translation artifact '{rejected}'"
                ));
            }
        }
    }
}

fn validate_cohesion(
    story: &StoryAsset,
    target_tokens: &HashSet<String>,
    field: &str,
    issues: &mut Vec<String>,
) {
    let stop_words = [
        "a", "an", "and", "are", "as", "at", "be", "been", "but", "by", "can", "did",
        "do", "for", "from", "had", "has", "have", "he", "her", "him", "his", "i", "in",
        "is", "it", "its", "of", "on", "or", "she", "that", "the", "their", "them", "there",
        "they", "this", "to", "was", "we", "were", "with", "you", "your",
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    let connectors = [
        "after", "although", "because", "before", "but", "however", "if", "so", "then",
        "though", "when", "while",
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    let character_tokens = story
        .characters
        .iter()
        .flat_map(|name| tokenize(name))
        .collect::<HashSet<_>>();
    let sentence_tokens = story
        .sentences
        .iter()
        .map(|sentence| tokenize(sentence))
        .collect::<Vec<_>>();

    let mut frequencies = HashMap::<String, usize>::new();
    for tokens in &sentence_tokens {
        for token in tokens {
            if !stop_words.contains(token.as_str())
                && !connectors.contains(token.as_str())
                && !character_tokens.contains(token)
                && !target_tokens.contains(token)
            {
                *frequencies.entry(token.clone()).or_default() += 1;
            }
        }
    }
    let anchors = frequencies
        .into_iter()
        .filter_map(|(token, count)| (count >= 2).then_some(token))
        .collect::<HashSet<_>>();

    let linked_sentences = sentence_tokens
        .iter()
        .filter(|tokens| {
            tokens.iter().any(|token| {
                anchors.contains(token)
                    || character_tokens.contains(token)
                    || connectors.contains(token.as_str())
            })
        })
        .count();
    if linked_sentences * 4 < sentence_tokens.len() * 3 {
        issues.push(format!(
            "{field}.sentences: article lacks a persistent scene or entity chain; too many sentences read as isolated examples"
        ));
    }

    let mut linked_transitions = 0usize;
    for index in 1..sentence_tokens.len() {
        let previous = sentence_tokens[index - 1]
            .iter()
            .chain(index.checked_sub(2).into_iter().flat_map(|prior| sentence_tokens[prior].iter()))
            .collect::<HashSet<_>>();
        if sentence_tokens[index].iter().any(|token| {
            previous.contains(&token)
                || anchors.contains(token)
                || character_tokens.contains(token)
                || connectors.contains(token.as_str())
        }) {
            linked_transitions += 1;
        }
    }
    if linked_transitions * 4 < sentence_tokens.len().saturating_sub(1) * 3 {
        issues.push(format!(
            "{field}.sentences: too many adjacent sentences lack a causal or entity link"
        ));
    }

    if let Some(last_sentence) = sentence_tokens.last() {
        let earlier = sentence_tokens[..sentence_tokens.len().saturating_sub(2)]
            .iter()
            .flatten()
            .collect::<HashSet<_>>();
        if !last_sentence.iter().any(|token| {
            earlier.contains(&token)
                && !target_tokens.contains(token)
                && !connectors.contains(token.as_str())
                && !character_tokens.contains(token)
        }) {
            issues.push(format!(
                "{field}.sentences: resolution does not recall an earlier story element"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{Reading, Stage};

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
                "Tom and Anna wait on a chair."
            ),
            Some("汤姆和安娜坐在椅子上等待。")
        );
    }

    #[test]
    fn story_coverage_rejects_an_ordinary_lesson_without_a_curated_story() {
        let course = CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "oxford-a1".to_owned(),
                title: "Oxford A1".to_owned(),
                lessons: vec![Lesson {
                    id: "a1-missing-story".to_owned(),
                    title: "Missing story".to_owned(),
                    new_words: Vec::new(),
                    sentences: Vec::new(),
                    reading: Reading {
                        title: "Controlled context".to_owned(),
                        sentences: Vec::new(),
                        questions: Vec::new(),
                    },
                }],
            }],
        };

        assert_eq!(missing_story_ids(&course), vec!["a1-missing-story"]);
        let error = validate_story_bank_coverage(&course).expect_err("missing story");
        assert!(error.to_string().contains("a1-missing-story"));
    }

    #[test]
    fn curated_story_language_has_no_known_quality_artifacts() {
        for story in stories().values() {
            let mut issues = Vec::new();
            validate_story_language(story, &story.lesson_id, &mut issues);
            assert!(issues.is_empty(), "{}", issues.join(" | "));
        }
    }

    #[test]
    fn paragraph_ranges_fallback_cover_legacy_readings() {
        assert_eq!(fallback_paragraph_ranges(0), Vec::<(usize, usize)>::new());
        assert_eq!(fallback_paragraph_ranges(10), vec![(0, 5), (5, 10)]);
        assert_eq!(fallback_paragraph_ranges(16), vec![(0, 6), (6, 11), (11, 16)]);
    }
}
