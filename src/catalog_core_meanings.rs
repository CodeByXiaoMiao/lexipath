use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct CoreSenseRecord {
    word: String,
    meaning: String,
    phrase: String,
    first: String,
    second: String,
}

static RECORDS: OnceLock<HashMap<String, CoreSenseRecord>> = OnceLock::new();

pub fn core_sense_template(word: &str) -> Option<(String, String, String, String)> {
    records()
        .get(&word.to_ascii_lowercase())
        .map(|record| {
            (
                record.meaning.clone(),
                record.phrase.clone(),
                record.first.clone(),
                record.second.clone(),
            )
        })
}

fn records() -> &'static HashMap<String, CoreSenseRecord> {
    RECORDS.get_or_init(|| {
        let entries: Vec<CoreSenseRecord> = serde_json::from_str(include_str!(
            "../assets/course-templates/function-6.json"
        ))
        .expect("reviewed core-sense template data is invalid");

        let mut output = HashMap::new();
        for entry in entries {
            let key = entry.word.to_ascii_lowercase();
            assert!(
                output.insert(key.clone(), entry).is_none(),
                "duplicate reviewed core sense: {key}"
            );
        }
        output
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_reviewed_core_senses_load_once() {
        assert_eq!(records().len(), 71);
    }

    #[test]
    fn old_uses_the_common_adjective_sense() {
        let record = core_sense_template("old").expect("record");
        assert_eq!(record.0, "adj. 老的；旧的");
        assert_eq!(record.2, "It is old.");
    }
}
