use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::course::{CoursePack, WordItem};

const EXAMPLE_TRANSLATION_SCHEMA_VERSION: u32 = 1;
const MISSING_TRANSLATION_TEXT: &str = "（例句中文译文缺失）";
const BANK_FILES: &[&str] = &[
    include_str!("../assets/example-translations/foundation-words-01.tsv"),
    include_str!("../assets/example-translations/ogden-850-01.tsv"),
    include_str!("../assets/example-translations/ogden-850-02.tsv"),
    include_str!("../assets/example-translations/ogden-850-03-04.tsv"),
    include_str!("../assets/example-translations/oxford-a1-01-02.tsv"),
    include_str!("../assets/example-translations/oxford-a1-03-a2-01.tsv"),
    include_str!("../assets/example-translations/oxford-a2-02-03.tsv"),
    include_str!("../assets/example-translations/oxford-b1-01-02.tsv"),
    include_str!("../assets/example-translations/oxford-b1-03-b2-01.tsv"),
    include_str!("../assets/example-translations/oxford-b2-02-03.tsv"),
];
const CORRECTION_FILES: &[&str] = &[
    include_str!("../assets/example-translations/review-corrections.tsv"),
    include_str!("../assets/example-translations/review-corrections-a1-01.tsv"),
    include_str!("../assets/example-translations/review-corrections-a1-02.tsv"),
    include_str!("../assets/example-translations/review-corrections-a1-03.tsv"),
    include_str!("../assets/example-translations/review-corrections-a1-04.tsv"),
    include_str!("../assets/example-translations/review-corrections-a2.tsv"),
    include_str!("../assets/example-translations/review-corrections-a2-z-final.tsv"),
];

#[derive(Debug, Clone)]
struct ExampleTranslationRecord {
    word_id: String,
    english_hash: u64,
    chinese: String,
}

static RECORDS: OnceLock<Vec<ExampleTranslationRecord>> = OnceLock::new();
static INDEX: OnceLock<HashMap<String, ExampleTranslationRecord>> = OnceLock::new();

pub fn reviewed_example_translation(word: &WordItem) -> Option<&'static str> {
    let record = index().get(&word.id)?;
    (record.english_hash == fnv1a64(&word.example)).then_some(record.chinese.as_str())
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

    let mut course_ids = HashSet::new();
    for stage in &course.stages {
        for lesson in &stage.lessons {
            for word in &lesson.new_words {
                course_ids.insert(word.id.as_str());
                let field = format!("lesson {} / word {}", lesson.id, word.id);
                let actual_hash = fnv1a64(&word.example);
                match index().get(&word.id) {
                    None => issues.push(format!(
                        "{field}: reviewed Chinese example translation is missing"
                    )),
                    Some(record) if record.english_hash != actual_hash => issues.push(format!(
                        "{field}: translation fingerprint mismatch; bank={:016x}, expected={actual_hash:016x}, example='{}'",
                        record.english_hash, word.example
                    )),
                    Some(record) => validate_chinese(&field, &record.chinese, &mut issues),
                }
            }
        }
    }
    for record in records() {
        if !course_ids.contains(record.word_id.as_str()) {
            issues.push(format!(
                "translation record '{}' has no matching course word",
                record.word_id
            ));
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "reviewed example translation validation failed: {}",
            issues.into_iter().take(300).collect::<Vec<_>>().join(" | ")
        )
    }
}

fn records() -> &'static Vec<ExampleTranslationRecord> {
    RECORDS.get_or_init(|| {
        let mut output = Vec::new();
        for content in BANK_FILES {
            parse_bank_file(content, "direct-llm-reviewed", &mut output);
        }

        for content in CORRECTION_FILES {
            let mut corrections = Vec::new();
            parse_bank_file(content, "direct-llm-correction", &mut corrections);
            let mut file_ids = HashSet::new();
            for correction in corrections {
                assert!(
                    file_ids.insert(correction.word_id.clone()),
                    "duplicate reviewed correction in one layer for '{}'",
                    correction.word_id
                );
                let target = output
                    .iter_mut()
                    .find(|record| record.word_id == correction.word_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "reviewed correction has no base record for '{}'",
                            correction.word_id
                        )
                    });
                *target = correction;
            }
        }
        output
    })
}

fn parse_bank_file(
    content: &str,
    expected_authoring_method: &str,
    output: &mut Vec<ExampleTranslationRecord>,
) {
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
        format!("authoring_method\t{expected_authoring_method}"),
        "unsupported example translation authoring method"
    );

    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let word_id = fields.next().unwrap_or_default();
        let english_hash = fields.next().unwrap_or_default();
        let chinese = fields.next().unwrap_or_default();
        assert!(
            !word_id.is_empty() && !english_hash.is_empty() && !chinese.is_empty(),
            "invalid example translation record at data line {}",
            index + 3
        );
        output.push(ExampleTranslationRecord {
            word_id: word_id.to_owned(),
            english_hash: u64::from_str_radix(english_hash, 16)
                .expect("invalid example translation English fingerprint"),
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

fn fnv1a64(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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
        "这是一十亿。",
        "这是几把剪刀。",
        "他是女性。",
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

    #[test]
    fn reviewed_corrections_override_base_candidates() {
        let female = WordItem {
            id: "ogden-female".to_owned(),
            text: "female".to_owned(),
            ipa: "/test/".to_owned(),
            meaning: "adj. 女性的".to_owned(),
            phrase: "be female".to_owned(),
            example: "She is female.".to_owned(),
        };
        assert_eq!(reviewed_example_translation(&female), Some("她是女性。"));

        let scissors = WordItem {
            id: "ogden-scissors".to_owned(),
            text: "scissors".to_owned(),
            ipa: "/test/".to_owned(),
            meaning: "n. 剪刀".to_owned(),
            phrase: "scissors".to_owned(),
            example: "These are scissors.".to_owned(),
        };
        assert_eq!(reviewed_example_translation(&scissors), Some("这是一把剪刀。"));
    }

    #[test]
    fn changed_english_never_reuses_a_stale_translation() {
        let word = WordItem {
            id: "w-a".to_owned(),
            text: "a".to_owned(),
            ipa: "/ə/".to_owned(),
            meaning: "一个".to_owned(),
            phrase: "a book".to_owned(),
            example: "This is a different example.".to_owned(),
        };
        assert_eq!(reviewed_example_translation(&word), None);
    }
}
