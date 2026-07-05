use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::course::{CoursePack, Lesson};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub lesson_id: String,
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lesson {} / {}: {}", self.lesson_id, self.field, self.message)
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() || (ch == '\'' && !current.is_empty()) {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(current.trim_matches('\'').to_owned());
            current.clear();
        }
    }

    if !current.is_empty() {
        tokens.push(current.trim_matches('\'').to_owned());
    }

    tokens.into_iter().filter(|token| !token.is_empty()).collect()
}

pub fn validate_course(course: &CoursePack) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut learned_words = HashSet::<String>::new();
    let mut lesson_ids = HashSet::<String>::new();

    for stage in &course.stages {
        for lesson in &stage.lessons {
            if !lesson_ids.insert(lesson.id.clone()) {
                errors.push(error(lesson, "id", "duplicate lesson id"));
            }

            let mut current_words = HashSet::new();
            for word in &lesson.new_words {
                let normalized = word.text.to_ascii_lowercase();
                if tokenize(&word.text).len() != 1 {
                    errors.push(error(
                        lesson,
                        "new_words",
                        &format!("word '{}' must contain exactly one token", word.text),
                    ));
                }
                if !current_words.insert(normalized.clone()) || learned_words.contains(&normalized) {
                    errors.push(error(
                        lesson,
                        "new_words",
                        &format!("word '{}' is duplicated in the course path", word.text),
                    ));
                }
            }

            learned_words.extend(current_words.iter().cloned());
            validate_lesson_text(lesson, &learned_words, &mut errors);
            validate_reading_coverage(lesson, &current_words, &mut errors);

            for (index, question) in lesson.reading.questions.iter().enumerate() {
                if question.options.is_empty() || question.correct_index >= question.options.len() {
                    errors.push(error(
                        lesson,
                        &format!("reading.questions[{index}]"),
                        "correct_index is outside the options array",
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_lesson_text(
    lesson: &Lesson,
    allowed: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    for (index, word) in lesson.new_words.iter().enumerate() {
        validate_text(lesson, &format!("new_words[{index}].phrase"), &word.phrase, allowed, errors);
        validate_text(lesson, &format!("new_words[{index}].example"), &word.example, allowed, errors);
    }

    for (index, sentence) in lesson.sentences.iter().enumerate() {
        validate_text(lesson, &format!("sentences[{index}]"), &sentence.text, allowed, errors);
    }

    for (index, sentence) in lesson.reading.sentences.iter().enumerate() {
        validate_text(
            lesson,
            &format!("reading.sentences[{index}]"),
            sentence,
            allowed,
            errors,
        );
    }

    for (question_index, question) in lesson.reading.questions.iter().enumerate() {
        for (option_index, option) in question.options.iter().enumerate() {
            validate_text(
                lesson,
                &format!("reading.questions[{question_index}].options[{option_index}]"),
                option,
                allowed,
                errors,
            );
        }
    }
}

fn validate_text(
    lesson: &Lesson,
    field: &str,
    text: &str,
    allowed: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if text.trim().is_empty() {
        errors.push(error(lesson, field, "English text cannot be empty"));
        return;
    }

    let unknown: Vec<String> = tokenize(text)
        .into_iter()
        .filter(|token| !allowed.contains(token))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if !unknown.is_empty() {
        errors.push(error(
            lesson,
            field,
            &format!("contains words outside the learned whitelist: {}", unknown.join(", ")),
        ));
    }
}

fn validate_reading_coverage(
    lesson: &Lesson,
    current_words: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    let counts = tokenize(&lesson.full_reading_text())
        .into_iter()
        .fold(HashMap::<String, usize>::new(), |mut counts, token| {
            *counts.entry(token).or_default() += 1;
            counts
        });

    for word in current_words {
        let count = counts.get(word).copied().unwrap_or_default();
        if count < 2 {
            errors.push(error(
                lesson,
                "reading",
                &format!("new word '{word}' must appear at least twice; found {count}"),
            ));
        }
    }
}

fn error(lesson: &Lesson, field: &str, message: &str) -> ValidationError {
    ValidationError {
        lesson_id: lesson.id.clone(),
        field: field.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::CoursePack;

    #[test]
    fn embedded_course_has_zero_unknown_words() {
        let course = CoursePack::embedded().expect("embedded course should parse");
        assert_eq!(validate_course(&course), Ok(()));
    }

    #[test]
    fn unknown_reading_word_is_rejected() {
        let mut course = CoursePack::embedded().expect("embedded course should parse");
        course.stages[0].lessons[0]
            .reading
            .sentences
            .push("I am outside.".to_owned());

        let errors = validate_course(&course).expect_err("unknown word must fail validation");
        assert!(errors.iter().any(|item| item.message.contains("outside")));
    }
}
