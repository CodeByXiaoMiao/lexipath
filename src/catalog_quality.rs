use std::collections::HashSet;
use std::fmt;

use crate::catalog_stories::has_curated_story;
use crate::controlled_english::{infer_morph_class, sequence_count};
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
                validate_ipa(lesson, &format!("{field}.ipa"), &word.ipa, &mut issues);

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
                let morph_class = infer_morph_class(
                    &word.text,
                    &word.meaning,
                    &word.phrase,
                    &word.example,
                );
                let containing_sentences = lesson
                    .reading
                    .sentences
                    .iter()
                    .filter(|sentence| {
                        sequence_count(&tokenize(sentence), &entry_tokens, morph_class) > 0
                    })
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
    if !has_curated_story(&lesson.id)
        && lesson.reading.sentences.len() < lesson.new_words.len() * 2
    {
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
        let enforce_controlled_length =
            !field.starts_with("reading.sentences[") || !has_curated_story(&lesson.id);
        if enforce_controlled_length {
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
}

fn validate_ipa(lesson: &Lesson, field: &str, ipa: &str, issues: &mut Vec<QualityIssue>) {
    if ipa.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '?' | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'
            )
            || character == '\u{fffd}'
    }) {
        issues.push(issue(
            lesson,
            field,
            "phonetic transcription contains an unknown, invisible, or control character",
        ));
    }
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

    #[test]
    fn rejects_corrupted_ipa_markers() {
        let mut course = course_with_word("thing", "This thing is useful.");
        course.stages[0].lessons[0].new_words[0].ipa = format!("/ˈθ{}ŋ/", '\u{fffd}');
        let errors = validate_content_quality(&course).expect_err("corrupted IPA must fail");
        assert!(errors
            .iter()
            .any(|error| error.message.contains("unknown, invisible, or control")));
    }
}
