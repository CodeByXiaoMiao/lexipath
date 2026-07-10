use std::collections::HashSet;

use crate::catalog_context_repairs::apply_context_repairs;
use crate::catalog_meaning::{learner_gloss, normalize_learner_meaning};
use crate::catalog_stories::{apply_curated_story, has_curated_story, validate_curated_story};
use crate::course::{CoursePack, Lesson, Question};
use crate::validator::tokenize;

const CONTROLLED_CONTEXT_TITLE: &str = "本课受控语境与完形练习";

pub fn formalize_generated_lessons(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id == "foundation-words" {
            continue;
        }
        for lesson in &mut stage.lessons {
            if lesson.is_stage_assessment() {
                continue;
            }

            apply_context_repairs(lesson);
            for (index, word) in lesson.new_words.iter_mut().enumerate() {
                word.meaning = normalize_learner_meaning(
                    &word.text,
                    &word.meaning,
                    &word.phrase,
                    &word.example,
                );
                if let Some(sentence) = lesson.sentences.get_mut(index) {
                    sentence.meaning = word.meaning.clone();
                }
            }
            if !apply_curated_story(lesson) {
                lesson.reading.title = CONTROLLED_CONTEXT_TITLE.to_owned();
            }
            lesson.reading.questions = build_cloze_questions(lesson);
        }
    }
}

pub fn validate_formalized_course(course: &CoursePack) -> anyhow::Result<()> {
    let mut issues = Vec::<String>::new();

    for stage in &course.stages {
        for lesson in &stage.lessons {
            if stage.id == "foundation-words" || lesson.is_stage_assessment() {
                continue;
            }

            validate_lesson(lesson, &mut issues);
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "formal course validation failed: {}",
            issues.into_iter().take(120).collect::<Vec<_>>().join(" | ")
        )
    }
}

fn validate_lesson(lesson: &Lesson, issues: &mut Vec<String>) {
    if has_curated_story(&lesson.id) {
        issues.extend(validate_curated_story(lesson));
    } else if lesson.reading.title != CONTROLLED_CONTEXT_TITLE {
        issues.push(format!(
            "lesson {} / reading.title: expected controlled-context practice title",
            lesson.id
        ));
    }
    if lesson.reading.questions.len() != lesson.new_words.len() {
        issues.push(format!(
            "lesson {} / reading.questions: expected {} cloze questions, found {}",
            lesson.id,
            lesson.new_words.len(),
            lesson.reading.questions.len()
        ));
    }

    let expected_options = lesson
        .new_words
        .iter()
        .map(|word| word.text.as_str())
        .collect::<HashSet<_>>();

    for (index, word) in lesson.new_words.iter().enumerate() {
        validate_meaning(&lesson.id, index, &word.meaning, issues);
        if let Some(sentence) = lesson.sentences.get(index) {
            if sentence.meaning != word.meaning {
                issues.push(format!(
                    "lesson {} / sentences[{index}].meaning: does not match the normalized target meaning",
                    lesson.id
                ));
            }
        }
        if word.example.contains("say \"") && !word.meaning.starts_with("interj. ") {
            issues.push(format!(
                "lesson {} / new_words[{index}].example: non-interjection uses a metalinguistic quotation",
                lesson.id
            ));
        }
        let Some(question) = lesson.reading.questions.get(index) else {
            continue;
        };
        let field = format!("lesson {} / reading.questions[{index}]", lesson.id);

        if !question.prompt.contains("____") {
            issues.push(format!("{field}: prompt has no cloze blank"));
        }
        if contains_sequence(&tokenize(&question.prompt), &tokenize(&word.text)) {
            issues.push(format!("{field}: prompt leaks target entry '{}'", word.text));
        }

        let actual_options = question
            .options
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if question.options.len() < 2
            || question.options.len() != lesson.new_words.len()
            || actual_options != expected_options
        {
            issues.push(format!(
                "{field}: options must be the unique new entries from this lesson"
            ));
        }
        match question.options.get(question.correct_index) {
            Some(correct) if correct == &word.text => {}
            _ => issues.push(format!(
                "{field}: correct option does not match target entry '{}'",
                word.text
            )),
        }
    }
}

fn validate_meaning(lesson_id: &str, index: usize, meaning: &str, issues: &mut Vec<String>) {
    let field = format!("lesson {lesson_id} / new_words[{index}].meaning");
    let trimmed = meaning.trim();
    if trimmed.contains("\\n")
        || trimmed.contains('\n')
        || trimmed.contains('[')
        || trimmed.contains(']')
    {
        issues.push(format!("{field}: raw dictionary metadata remains"));
    }
    if trimmed.contains(',') || trimmed.contains('，') {
        issues.push(format!("{field}: more than one comma-separated sense remains"));
    }
    let normalized_label = [
        "n. ", "v. ", "adj. ", "adv. ", "prep. ", "conj. ", "pron. ",
        "det. ", "modal. ", "num. ", "interj. ", "word. ",
    ]
    .iter()
    .any(|prefix| trimmed.starts_with(prefix));
    if !normalized_label {
        issues.push(format!("{field}: normalized part-of-speech label is missing"));
    }
}

fn build_cloze_questions(lesson: &Lesson) -> Vec<Question> {
    let entries = lesson
        .new_words
        .iter()
        .map(|word| word.text.clone())
        .collect::<Vec<_>>();

    lesson
        .new_words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            let option_count = entries.len();
            let shift = if option_count == 0 {
                0
            } else {
                (index * 2 + 1) % option_count
            };
            let options = (0..option_count)
                .map(|offset| entries[(shift + offset) % option_count].clone())
                .collect::<Vec<_>>();
            let correct_index = if option_count == 0 {
                0
            } else {
                (index + option_count - shift) % option_count
            };
            Question {
                prompt: format!(
                    "根据“{}”完成句子：{}",
                    learner_gloss(&word.meaning),
                    cloze_sentence(question_source_sentence(lesson, word, index), &word.text)
                ),
                options,
                correct_index,
            }
        })
        .collect()
}

fn question_source_sentence<'a>(
    lesson: &'a Lesson,
    word: &'a crate::course::WordItem,
    index: usize,
) -> &'a str {
    let entry_tokens = tokenize(&word.text);
    let matches = lesson
        .reading
        .sentences
        .iter()
        .filter(|sentence| contains_sequence(&tokenize(sentence), &entry_tokens))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        &word.example
    } else {
        matches[index % matches.len()].as_str()
    }
}

fn cloze_sentence(sentence: &str, entry: &str) -> String {
    let sentence_lower = sentence.to_ascii_lowercase();
    let entry_lower = entry.to_ascii_lowercase();
    for (start, _) in sentence_lower.match_indices(&entry_lower) {
        let end = start + entry_lower.len();
        let left_is_word = start > 0
            && sentence_lower[..start]
                .chars()
                .next_back()
                .map(is_word_character)
                .unwrap_or(false);
        let right_is_word = end < sentence_lower.len()
            && sentence_lower[end..]
                .chars()
                .next()
                .map(is_word_character)
                .unwrap_or(false);
        if !left_is_word && !right_is_word {
            let mut output = sentence.to_owned();
            output.replace_range(start..end, "____");
            return output;
        }
    }
    sentence.to_owned()
}

fn is_word_character(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '\''
}

fn contains_sequence(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{Reading, SentenceItem, Stage, WordItem};

    #[test]
    fn cloze_questions_hide_the_answer_and_rotate_options() {
        let lesson = Lesson {
            id: "unit".to_owned(),
            title: "unit".to_owned(),
            new_words: vec![
                word("book", "n. 书", "This is a book."),
                word("room", "n. 房间", "This is a room."),
            ],
            sentences: vec![SentenceItem {
                text: "This is a book.".to_owned(),
                meaning: "n. 书".to_owned(),
            }],
            reading: Reading {
                title: String::new(),
                sentences: Vec::new(),
                questions: Vec::new(),
            },
        };
        let questions = build_cloze_questions(&lesson);
        assert!(questions[0].prompt.contains("____"));
        assert!(!questions[0].prompt.to_ascii_lowercase().contains("book"));
        assert_eq!(questions[0].options[questions[0].correct_index], "book");
    }

    fn word(text: &str, meaning: &str, example: &str) -> WordItem {
        WordItem {
            id: text.to_owned(),
            text: text.to_owned(),
            ipa: "/test/".to_owned(),
            meaning: meaning.to_owned(),
            phrase: text.to_owned(),
            example: example.to_owned(),
        }
    }

    #[allow(dead_code)]
    fn course(lesson: Lesson) -> CoursePack {
        CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "generated".to_owned(),
                title: "generated".to_owned(),
                lessons: vec![lesson],
            }],
        }
    }
}
