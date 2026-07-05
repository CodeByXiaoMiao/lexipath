use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct TemplateRecord {
    word: String,
    phrase: String,
    first: String,
    second: String,
}

static TEMPLATES: OnceLock<HashMap<String, TemplateRecord>> = OnceLock::new();

pub fn function_template(word: &str) -> Option<(String, String, String)> {
    templates()
        .get(&word.to_ascii_lowercase())
        .map(|record| {
            (
                record.phrase.clone(),
                record.first.clone(),
                record.second.clone(),
            )
        })
}

fn templates() -> &'static HashMap<String, TemplateRecord> {
    TEMPLATES.get_or_init(|| {
        let mut output = HashMap::new();
        for source in [
            include_str!("../assets/course-templates/function-1.json"),
            include_str!("../assets/course-templates/function-2.json"),
            include_str!("../assets/course-templates/function-3.json"),
            include_str!("../assets/course-templates/function-4.json"),
            include_str!("../assets/course-templates/function-5.json"),
        ] {
            let records: Vec<TemplateRecord> =
                serde_json::from_str(source).expect("reviewed function template data is invalid");
            for record in records {
                let key = record.word.to_ascii_lowercase();
                assert!(
                    output.insert(key.clone(), record).is_none(),
                    "duplicate reviewed function template: {key}"
                );
            }
        }
        output
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_reviewed_templates_are_loaded_once() {
        assert_eq!(templates().len(), 261);
    }

    #[test]
    fn pronouns_use_a_real_sentence_instead_of_metalinguistic_fallback() {
        let template = function_template("my").expect("template");
        assert_eq!(template.1, "My book is here.");
    }

    #[test]
    fn irregular_plural_nouns_use_plural_agreement() {
        let template = function_template("bacteria").expect("template");
        assert_eq!(template.1, "These are bacteria.");
    }

    #[test]
    fn unit_uses_a_because_it_begins_with_a_y_sound() {
        let template = function_template("unit").expect("template");
        assert_eq!(template.1, "This is a unit.");
    }
}
