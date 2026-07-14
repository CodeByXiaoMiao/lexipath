use std::sync::OnceLock;

const FINAL_FREEZE_TEMPLATE_FILE: &str =
    include_str!("../assets/example-templates/final-freeze.tsv");

#[derive(Debug)]
struct FinalFreezeTemplate {
    word_id: &'static str,
    display: Option<&'static str>,
    meaning: Option<&'static str>,
    phrase: &'static str,
    primary_example: &'static str,
    secondary_example: &'static str,
}

static RECORDS: OnceLock<Vec<FinalFreezeTemplate>> = OnceLock::new();

pub fn final_freeze_template(
    word_id: &str,
) -> Option<(Option<String>, Option<String>, String, String, String)> {
    let record = records().iter().find(|record| record.word_id == word_id)?;
    Some((
        record.display.map(str::to_owned),
        record.meaning.map(str::to_owned),
        record.phrase.to_owned(),
        record.primary_example.to_owned(),
        record.secondary_example.to_owned(),
    ))
}

fn records() -> &'static Vec<FinalFreezeTemplate> {
    RECORDS.get_or_init(|| parse_file(FINAL_FREEZE_TEMPLATE_FILE))
}

fn parse_file(content: &'static str) -> Vec<FinalFreezeTemplate> {
    let mut lines = content.lines();
    assert_eq!(lines.next(), Some("# schema=1"));
    assert_eq!(
        lines.next(),
        Some("authoring_method\tdirect-assistant-reviewed")
    );
    assert_eq!(
        lines.next(),
        Some("word_id\tdisplay\tmeaning\tphrase\tprimary_example\tsecondary_example")
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let word_id = required(fields.next(), "word_id");
            let display = optional(fields.next());
            let meaning = optional(fields.next());
            let phrase = required(fields.next(), "phrase");
            let primary_example = required(fields.next(), "primary_example");
            let secondary_example = required(fields.next(), "secondary_example");
            assert!(
                fields.next().is_none(),
                "too many final-freeze template fields"
            );
            FinalFreezeTemplate {
                word_id,
                display,
                meaning,
                phrase,
                primary_example,
                secondary_example,
            }
        })
        .collect()
}

fn required(value: Option<&'static str>, name: &str) -> &'static str {
    let value = value.unwrap_or_else(|| panic!("missing final-freeze template field '{name}'"));
    assert!(
        !value.is_empty(),
        "empty final-freeze template field '{name}'"
    );
    value
}

fn optional(value: Option<&'static str>) -> Option<&'static str> {
    value.filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn final_freeze_template_bank_is_unique() {
        assert_eq!(records().len(), 91);
        let mut ids = HashSet::new();
        for record in records() {
            assert!(ids.insert(record.word_id), "{}", record.word_id);
        }
    }

    #[test]
    fn contains_cross_stage_freeze_fixes() {
        let voice = final_freeze_template("ogden-voice").expect("voice template");
        assert_eq!(voice.1.as_deref(), Some("n. 嗓音；人声"));
        assert_eq!(voice.3, "The man's voice is clear.");

        let background =
            final_freeze_template("a2-background").expect("background template");
        assert_eq!(background.3, "The background is blue.");

        let studio = final_freeze_template("b1-studio").expect("studio template");
        assert_eq!(studio.3, "The film is from a small studio.");

        let slight = final_freeze_template("b2-slight").expect("slight template");
        assert_eq!(slight.1.as_deref(), Some("n. 怠慢；轻视"));
        assert_eq!(slight.3, "I took his silence as a slight.");
    }
}
