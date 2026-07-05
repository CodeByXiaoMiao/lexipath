use std::collections::HashSet;
use std::fmt;

use crate::course::{CoursePack, Lesson};
use crate::validator::tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityIssue {
    pub lesson_id: String,
    pub field: String,
    pub message: String,
}

impl fmt::Display for QualityIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "lesson {} / {}: {}",
            self.lesson_id, self.field, self.message
        )
    }
}

pub fn validate_content_quality(course: &CoursePack) -> Result<(), Vec<QualityIssue>> {
    let mut issues = Vec::new();
    let mut generated_word_count = 0usize;
    let mut fallback_count = 0usize;

    for stage in &course.stages {
        for lesson in &stage.lessons {
            validate_lesson_shape(lesson, &mut issues);

            for (index, word) in lesson.new_words.iter().enumerate() {
                if stage.id != "foundation-words" {
                    generated_word_count += 1;
                }
                if word.example.starts_with("The word is ") {
                    fallback_count += 1;
                }

                let field = format!("new_words[{index}]");
                if word.text.trim().is_empty()
                    || word.text.starts_with('-')
                    || word.text.ends_with('-')
                    || word.text.ends_with('.')
                {
                    issues.push(issue(
                        lesson,
                        &field,
                        &format!("invalid learner-facing entry form: '{}'", word.text),
                    ));
                }

                let meaning = word.meaning.trim().to_ascii_lowercase();
                if meaning.starts_with("suf.")
                    || meaning.starts_with("comb.")
                    || meaning.starts_with("abbr.")
                    || meaning.contains("[构成")
                {
                    issues.push(issue(
                        lesson,
                        &format!("{field}.meaning"),
                        "dictionary affix or abbreviation metadata leaked into the course",
                    ));
                }

                if word.ipa.trim_matches('/').trim().is_empty() {
                    issues.push(issue(
                        lesson,
                        &format!("{field}.ipa"),
                        "phonetic transcription is empty",
                    ));
                }

                validate_sentence(
                    lesson,
                    &format!("{field}.example"),
                    &word.example,
                    &mut issues,
                );
                validate_sentence(
                    lesson,
                    &format!("{field}.phrase"),
                    &word.phrase,
                    &mut issues,
                );

                let entry_tokens = tokenize(&word.text);
                let containing_sentences = lesson
                    .reading
                    .sentences
                    .iter()
                    .filter(|sentence| contains_sequence(&tokenize(sentence), &entry_tokens))
                    .count();
                if containing_sentences < 2 {
                    issues.push(issue(
                        lesson,
                        "reading",
                        &format!(
                            "entry '{}' must be used in at least two separate reading sentences",
                            word.text
                        ),
                    ));
                }
            }

            for (index, sentence) in lesson.reading.sentences.iter().enumerate() {
                validate_sentence(
                    lesson,
                    &format!("reading.sentences[{index}]"),
                    sentence,
                    &mut issues,
                );
            }

            for (question_index, question) in lesson.reading.questions.iter().enumerate() {
                let unique_options = question.options.iter().collect::<HashSet<_>>();
                if unique_options.len() != question.options.len() {
                    issues.push(issue(
                        lesson,
                        &format!("reading.questions[{question_index}]"),
                        "answer options are not unique",
                    ));
                }
            }
        }
    }

    let fallback_limit = (generated_word_count / 100).max(5);
    if fallback_count > fallback_limit {
        issues.push(QualityIssue {
            lesson_id: "course".to_owned(),
            field: "generated_content".to_owned(),
            message: format!(
                "metalinguistic fallback was used {fallback_count} times; limit is {fallback_limit}"
            ),
        });
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

fn validate_lesson_shape(lesson: &Lesson, issues: &mut Vec<QualityIssue>) {
    if lesson.new_words.is_empty() || lesson.new_words.len() > 6 {
        issues.push(issue(
            lesson,
            "new_words",
            "a learning unit must contain between one and six new entries",
        ));
    }
    if lesson.sentences.len() != lesson.new_words.len() {
        issues.push(issue(
            lesson,
            "sentences",
            "every new entry must have one sentence exercise",
        ));
    }
    if lesson.reading.sentences.len() < lesson.new_words.len() * 2 {
        issues.push(issue(
            lesson,
            "reading.sentences",
            "controlled reading must provide at least two contexts per new entry",
        ));
    }
    if lesson.reading.questions.len() != lesson.new_words.len() {
        issues.push(issue(
            lesson,
            "reading.questions",
            "every new entry must have one comprehension check",
        ));
    }
}

fn validate_sentence(
    lesson: &Lesson,
    field: &str,
    sentence: &str,
    issues: &mut Vec<QualityIssue>,
) {
    let trimmed = sentence.trim();
    if trimmed.is_empty() {
        issues.push(issue(lesson, field, "text is empty"));
        return;
    }
    if trimmed.contains("..") || trimmed.contains("?.") || trimmed.contains("!.") {
        issues.push(issue(lesson, field, "text contains malformed punctuation"));
    }
    if field.contains("sentence") || field.contains("example") {
        if !trimmed.ends_with('.') && !trimmed.ends_with('?') && !trimmed.ends_with('!') {
            issues.push(issue(lesson, field, "sentence has no terminal punctuation"));
        }
        let token_count = tokenize(trimmed).len();
        if token_count > 16 {
            issues.push(issue(
                lesson,
                field,
                &format!("controlled sentence is too long: {token_count} tokens"),
            ));
        }
    }
}

fn contains_sequence(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn issue(lesson: &Lesson, field: &str, message: &str) -> QualityIssue {
    QualityIssue {
        lesson_id: lesson.id.clone(),
        field: field.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_polish::polish_generated_content;
    use crate::embedded_course;

    #[test]
    fn polished_foundation_course_passes_the_quality_gate() {
        let mut course = embedded_course::load().expect("course");
        polish_generated_content(&mut course);
        assert_eq!(validate_content_quality(&course), Ok(()));
    }

    #[test]
    fn suffix_dictionary_entries_are_rejected() {
        let mut course = embedded_course::load().expect("course");
        course.stages[0].lessons[0].new_words[0].text = "-word".to_owned();
        assert!(validate_content_quality(&course).is_err());
    }
}
