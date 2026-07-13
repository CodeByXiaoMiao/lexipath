use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::course::{CoursePack, WordItem};

const EXAMPLE_TRANSLATION_SCHEMA_VERSION: u32 = 1;
const MISSING_TRANSLATION_TEXT: &str = "（例句中文译文缺失）";
const BANK_FILES: &[&str] = &[
    include_str!("../assets/example-translations/foundation-words.tsv"),
    include_str!("../assets/example-translations/ogden-850.tsv"),
    include_str!("../assets/example-translations/oxford-a1.tsv"),
    include_str!("../assets/example-translations/oxford-a2.tsv"),
    include_str!("../assets/example-translations/oxford-b1.tsv"),
    include_str!("../assets/example-translations/oxford-b2.tsv"),
];

#[derive(Debug, Clone)]
struct ExampleTranslationRecord {
    word_id: String,
    english: String,
    chinese: String,
}

static RECORDS: OnceLock<Vec<ExampleTranslationRecord>> = OnceLock::new();
static INDEX: OnceLock<HashMap<String, ExampleTranslationRecord>> = OnceLock::new();

pub fn reviewed_example_translation(word: &WordItem) -> Option<&'static str> {
    let record = index().get(&word.id)?;
    (record.english == word.example).then_some(record.chinese.as_str())
}

pub fn example_translation_or_error(word: &WordItem) -> String {
    reviewed_example_translation(word)
        .unwrap_or(MISSING_TRANSLATION_TEXT)
        .to_owned()
}

pub fn validate_example_translation_bank(course: &CoursePack) -> anyhow::Result<()> {
    let mut issues = Vec::new();
    let mut seen = HashSet::new();

    for record in records() {
        if !seen.insert(record.word_id.as_str()) {
            issues.push(format!(
                "duplicate example translation record for '{}'",
                record.word_id
            ));
        }
        validate_chinese(&record.word_id, &record.chinese, &mut issues);
    }

    for stage in &course.stages {
        for lesson in &stage.lessons {
            for word in &lesson.new_words {
                let field = format!("lesson {} / word {}", lesson.id, word.id);
                match index().get(&word.id) {
                    None => issues.push(format!(
                        "{field}: reviewed Chinese example translation is missing"
                    )),
                    Some(record) if record.english != word.example => issues.push(format!(
                        "{field}: translation bank English does not match the course example; bank='{}', course='{}'",
                        record.english, word.example
                    )),
                    Some(record) => validate_chinese(&field, &record.chinese, &mut issues),
                }
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "reviewed example translation validation failed: {}",
            issues.into_iter().take(120).collect::<Vec<_>>().join(" | ")
        )
    }
}

fn records() -> &'static Vec<ExampleTranslationRecord> {
    RECORDS.get_or_init(|| {
        let mut output = Vec::new();
        for content in BANK_FILES {
            parse_bank_file(content, &mut output);
        }
        output
    })
}

fn parse_bank_file(content: &str, output: &mut Vec<ExampleTranslationRecord>) {
    let mut lines = content.lines();
    let schema = lines.next().expect("translation bank file has no schema");
    assert_eq!(
        schema,
        format!("# schema={EXAMPLE_TRANSLATION_SCHEMA_VERSION}"),
        "unsupported example translation bank schema"
    );
    let authoring = lines
        .next()
        .expect("translation bank file has no authoring method");
    assert_eq!(
        authoring,
        "authoring_method\tdirect-llm-reviewed",
        "unsupported example translation authoring method"
    );

    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let word_id = fields.next().unwrap_or_default();
        let english = fields.next().unwrap_or_default();
        let chinese = fields.next().unwrap_or_default();
        assert!(
            !word_id.is_empty() && !english.is_empty() && !chinese.is_empty(),
            "invalid example translation record at data line {}",
            index + 3
        );
        output.push(ExampleTranslationRecord {
            word_id: word_id.to_owned(),
            english: english.to_owned(),
            chinese: chinese.to_owned(),
        });
    }
}

fn index() -> &'static HashMap<String, ExampleTranslationRecord> {
    INDEX.get_or_init(|| {
        let mut output = HashMap::new();
        for record in records() {
            output
                .entry(record.word_id.clone())
                .or_insert_with(|| record.clone());
        }
        output
    })
}

fn validate_chinese(field: &str, chinese: &str, issues: &mut Vec<String>) {
    let value = chinese.trim();
    if value.is_empty() {
        issues.push(format!("{field}: Chinese example translation is empty"));
        return;
    }
    if !value
        .chars()
        .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
    {
        issues.push(format!(
            "{field}: Chinese example translation contains no Chinese text"
        ));
    }
    if !(value.ends_with('。') || value.ends_with('！') || value.ends_with('？')) {
        issues.push(format!(
            "{field}: Chinese example translation needs Chinese sentence punctuation"
        ));
    }
    for artifact in [
        "这是一个书。",
        "这是一个食物。",
        "这是一 yard。",
        "例句中",
        "机器翻译",
        MISSING_TRANSLATION_TEXT,
    ] {
        if value.contains(artifact) {
            issues.push(format!(
                "{field}: Chinese example translation contains rejected artifact '{artifact}'"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_bank_has_unique_valid_records() {
        let mut seen = HashSet::new();
        assert!(records().len() >= 3_000);
        for record in records() {
            assert!(seen.insert(record.word_id.as_str()), "{}", record.word_id);
            let mut issues = Vec::new();
            validate_chinese(&record.word_id, &record.chinese, &mut issues);
            assert!(issues.is_empty(), "{}", issues.join(" | "));
        }
    }

    #[test]
    fn common_examples_use_natural_reviewed_chinese() {
        let word = WordItem {
            id: "w-a".to_owned(),
            text: "a".to_owned(),
            ipa: "/ə/".to_owned(),
            meaning: "一个".to_owned(),
            phrase: "a book".to_owned(),
            example: "This is a book.".to_owned(),
        };
        assert_eq!(
            reviewed_example_translation(&word),
            Some("这是一本书。")
        );
        assert_ne!(
            reviewed_example_translation(&word),
            Some("这是一个书。")
        );
    }
}
