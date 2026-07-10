use std::collections::HashSet;
use std::fmt;

use crate::course::{CoursePack, Lesson};
use crate::validator::tokenize;

const BAD_EXACT_EXAMPLES: &[&str] = &[
    "It is able.",
    "It is only.",
    "It is very.",
    "It is same.",
    "It is happy.",
    "It is interested.",
    "It is excited.",
    "It is surprised.",
    "It is shocked.",
    "It is unconscious.",
];

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
    let mut fallback_count = 0usize;

    for stage in &course.stages {
        let generated_stage = stage.id != "foundation-words";
        for lesson in &stage.lessons {
            if lesson.is_stage_assessment() {
                validate_stage_assessment(lesson, &mut issues);
            } else if generated_stage {
                validate_lesson_shape(lesson, &mut issues);
            }

            for (index, word) in lesson.new_words.iter().enumerate() {
                if generated_stage && word.example.starts_with("The word is ") {
                    fallback_count += 1;
                    issues.push(issue(
                        lesson,
                        &format!("new_words[{index}].example"),
                        "metalinguistic fallback example reached the final course",
                    ));
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

                if BAD_EXACT_EXAMPLES.contains(&word.example.as_str()) {
                    issues.push(issue(
                        lesson,
                        &format!("{field}.example"),
                        "manual review rejected this unnatural generated example",
                    ));
                }

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
                if BAD_EXACT_EXAMPLES.contains(&sentence.as_str()) {
                    issues.push(issue(
                        lesson,
                        &format!("reading.sentences[{index}]"),
                        "manual review rejected this unnatural generated sentence",
                    ));
                }
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

    if fallback_count > 0 {
        issues.push(QualityIssue {
            lesson_id: "course".to_owned(),
            field: "generated_content".to_owned(),
            message: format!("metalinguistic fallback was used {fallback_count} times; limit is 0"),
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

fn validate_stage_assessment(lesson: &Lesson, issues: &mut Vec<QualityIssue>) {
    if !lesson.new_words.is_empty() || !lesson.sentences.is_empty() {
        issues.push(issue(
            lesson,
            "assessment",
            "a stage assessment cannot introduce new words or sentence drills",
        ));
    }

    let reading_words = tokenize(&lesson.full_reading_text()).len();
    if !(800..=1_150).contains(&reading_words) {
        issues.push(issue(
            lesson,
            "reading",
            &format!(
                "stage final reading must contain 800 to 1150 learned words; found {reading_words}"
            ),
        ));
    }

    if !(15..=25).contains(&lesson.reading.questions.len()) {
        issues.push(issue(
            lesson,
            "reading.questions",
            &format!(
                "stage final assessment must contain 15 to 25 questions; found {}",
                lesson.reading.questions.len()
            ),
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
    use crate::course::{CoursePack, Lesson, Reading, Stage, WordItem};

    fn course_with_word(word: &str, example: &str) -> CoursePack {
        CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "generated".to_owned(),
                title: "generated".to_owned(),
                lessons: vec![Lesson {
                    id: "unit".to_owned(),
                    title: "unit".to_owned(),
                    new_words: vec![WordItem {
                        id: word.to_owned(),
                        text: word.to_owned(),
                        ipa: "/test/".to_owned(),
                        meaning: "n. test".to_owned(),
                        phrase: word.to_owned(),
                        example: example.to_owned(),
                    }],
                    sentences: vec![crate::course::SentenceItem {
                        text: example.to_owned(),
                        meaning: "test".to_owned(),
                    }],
                    reading: Reading {
                        title: "test".to_owned(),
                        sentences: vec![example.to_owned(), example.to_owned()],
                        questions: vec![crate::course::Question {
                            prompt: "test".to_owned(),
                            options: vec![example.to_owned()],
                            correct_index: 0,
                        }],
                    },
                }],
            }],
        }
    }

    #[test]
    fn rejects_metalinguistic_fallbacks() {
        let course = course_with_word("blog", "The word is blog.");
        let errors = validate_content_quality(&course).expect_err("fallback must fail");
        assert!(errors
            .iter()
            .any(|error| error.message.contains("metalinguistic fallback")));
    }

    #[test]
    fn rejects_known_bad_adjective_frames() {
        let course = course_with_word("happy", "It is happy.");
        let errors = validate_content_quality(&course).expect_err("bad frame must fail");
        assert!(errors
            .iter()
            .any(|error| error.message.contains("manual review rejected")));
    }
}
