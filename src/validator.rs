use std::collections::HashSet;
use std::fmt;

use crate::catalog_stories::allowed_character_tokens;
use crate::controlled_english::{
    infer_morph_class, sequence_count as controlled_sequence_count, surface_forms, MorphClass,
};
use crate::course::{CoursePack, Lesson};

pub use crate::controlled_english::tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub lesson_id: String,
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "lesson {} / {}: {}",
            self.lesson_id, self.field, self.message
        )
    }
}

#[derive(Debug, Default)]
struct AllowedLexicon {
    surfaces: HashSet<String>,
}

impl AllowedLexicon {
    fn learn_entry(&mut self, text: &str, meaning: &str, phrase: &str, example: &str) {
        let tokens = tokenize(text);
        self.surfaces.extend(tokens.iter().cloned());
        if let Some(last) = tokens.last() {
            self.surfaces.extend(surface_forms(
                last,
                infer_morph_class(text, meaning, phrase, example),
            ));
        }
    }

    fn contains(&self, token: &str) -> bool {
        self.surfaces.contains(token)
    }
}

pub fn validate_course(course: &CoursePack) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut learned_entries = HashSet::<String>::new();
    let mut learned = AllowedLexicon::default();
    let mut lesson_ids = HashSet::<String>::new();

    for stage in &course.stages {
        for lesson in &stage.lessons {
            if !lesson_ids.insert(lesson.id.clone()) {
                errors.push(error(lesson, "id", "duplicate lesson id"));
            }

            let mut current_entries = Vec::<(String, Vec<String>, MorphClass)>::new();
            for word in &lesson.new_words {
                let tokens = tokenize(&word.text);
                if tokens.is_empty() {
                    errors.push(error(
                        lesson,
                        "new_words",
                        &format!("entry '{}' contains no English token", word.text),
                    ));
                    continue;
                }

                let normalized = tokens.join(" ");
                if !learned_entries.insert(normalized) {
                    errors.push(error(
                        lesson,
                        "new_words",
                        &format!("entry '{}' is duplicated in the course path", word.text),
                    ));
                }

                learned.learn_entry(&word.text, &word.meaning, &word.phrase, &word.example);
                current_entries.push((
                    word.text.clone(),
                    tokens,
                    infer_morph_class(&word.text, &word.meaning, &word.phrase, &word.example),
                ));
            }

            validate_lesson_text(lesson, &learned, &mut errors);
            validate_reading_coverage(lesson, &current_entries, &mut errors);

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
    allowed: &AllowedLexicon,
    errors: &mut Vec<ValidationError>,
) {
    let no_names = HashSet::new();
    let reading_names = allowed_character_tokens(&lesson.id);

    for (index, word) in lesson.new_words.iter().enumerate() {
        validate_text(
            lesson,
            &format!("new_words[{index}].phrase"),
            &word.phrase,
            allowed,
            &no_names,
            errors,
        );
        validate_text(
            lesson,
            &format!("new_words[{index}].example"),
            &word.example,
            allowed,
            &no_names,
            errors,
        );
    }

    for (index, sentence) in lesson.sentences.iter().enumerate() {
        validate_text(
            lesson,
            &format!("sentences[{index}]"),
            &sentence.text,
            allowed,
            &no_names,
            errors,
        );
    }

    validate_text(
        lesson,
        "reading.title",
        &lesson.reading.title,
        allowed,
        &reading_names,
        errors,
    );

    for (index, sentence) in lesson.reading.sentences.iter().enumerate() {
        validate_text(
            lesson,
            &format!("reading.sentences[{index}]"),
            sentence,
            allowed,
            &reading_names,
            errors,
        );
    }

    for (question_index, question) in lesson.reading.questions.iter().enumerate() {
        validate_text(
            lesson,
            &format!("reading.questions[{question_index}].prompt"),
            &question.prompt,
            allowed,
            &reading_names,
            errors,
        );
        for (option_index, option) in question.options.iter().enumerate() {
            validate_text(
                lesson,
                &format!("reading.questions[{question_index}].options[{option_index}]"),
                option,
                allowed,
                &no_names,
                errors,
            );
        }
    }
}

fn validate_text(
    lesson: &Lesson,
    field: &str,
    text: &str,
    allowed: &AllowedLexicon,
    allowed_names: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if text.trim().is_empty() {
        errors.push(error(lesson, field, "English text cannot be empty"));
        return;
    }

    let mut unknown = tokenize(text)
        .into_iter()
        .filter(|token| !allowed.contains(token) && !allowed_names.contains(token))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    unknown.sort();

    if !unknown.is_empty() {
        errors.push(error(
            lesson,
            field,
            &format!(
                "contains words outside the learned whitelist: {}",
                unknown.join(", ")
            ),
        ));
    }
}

fn validate_reading_coverage(
    lesson: &Lesson,
    current_entries: &[(String, Vec<String>, MorphClass)],
    errors: &mut Vec<ValidationError>,
) {
    let reading_tokens = tokenize(&lesson.full_reading_text());

    for (display, tokens, class) in current_entries {
        let count = controlled_sequence_count(&reading_tokens, tokens, *class);
        if count < 2 {
            errors.push(error(
                lesson,
                "reading",
                &format!(
                    "new entry '{display}' must appear at least twice including controlled inflections; found {count}"
                ),
            ));
        }
        let exact_count = exact_sequence_count(&reading_tokens, tokens);
        if exact_count == 0 {
            errors.push(error(
                lesson,
                "reading",
                &format!(
                    "new entry '{display}' must appear at least once in its exact dictionary form"
                ),
            ));
        }
    }
}

fn exact_sequence_count(haystack: &[String], needle: &[String]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
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

    #[test]
    fn unknown_reading_title_word_is_rejected() {
        let mut course = CoursePack::embedded().expect("embedded course should parse");
        course.stages[0].lessons[0].reading.title = "Here and there".to_owned();

        let errors = validate_course(&course).expect_err("unknown title word must fail validation");
        assert!(errors.iter().any(|item| item.message.contains("and")));
    }

    #[test]
    fn phrase_coverage_uses_the_full_token_sequence() {
        let reading = tokenize("I am in front. You are in front.");
        let phrase = tokenize("in front");
        assert_eq!(exact_sequence_count(&reading, &phrase), 2);
    }

    #[test]
    fn learned_verb_allows_controlled_inflections() {
        let mut lexicon = AllowedLexicon::default();
        lexicon.learn_entry("plan", "v. 计划", "plan it", "I plan it.");
        assert!(lexicon.contains("plans"));
        assert!(lexicon.contains("planned"));
        assert!(lexicon.contains("planning"));
        assert!(!lexicon.contains("planet"));
    }

    #[test]
    fn possessive_name_tokenizes_as_the_declared_name() {
        assert_eq!(tokenize("Anna's advice"), ["anna", "advice"]);
    }
}
