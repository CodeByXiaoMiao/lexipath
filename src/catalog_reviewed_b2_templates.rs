use std::sync::OnceLock;

const REVIEWED_B2_TEMPLATE_FILES: &[&str] = &[
    include_str!("../assets/example-templates/oxford-b2-01.tsv"),
    include_str!("../assets/example-templates/oxford-b2-02.tsv"),
    include_str!("../assets/example-templates/oxford-b2-03.tsv"),
    include_str!("../assets/example-templates/oxford-b2-04.tsv"),
    include_str!("../assets/example-templates/oxford-b2-05.tsv"),
    include_str!("../assets/example-templates/oxford-b2-06.tsv"),
];

#[derive(Debug)]
struct ReviewedB2Template {
    word_id: &'static str,
    display: Option<&'static str>,
    meaning: Option<&'static str>,
    phrase: &'static str,
    primary_example: &'static str,
    secondary_example: &'static str,
}

static RECORDS: OnceLock<Vec<ReviewedB2Template>> = OnceLock::new();

pub fn reviewed_b2_template(
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

fn records() -> &'static Vec<ReviewedB2Template> {
    RECORDS.get_or_init(|| {
        REVIEWED_B2_TEMPLATE_FILES
            .iter()
            .flat_map(|content| parse_file(content))
            .collect()
    })
}

fn parse_file(content: &'static str) -> Vec<ReviewedB2Template> {
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
            assert!(fields.next().is_none(), "too many B2 template fields");
            ReviewedB2Template {
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
    let value = value.unwrap_or_else(|| panic!("missing B2 template field '{name}'"));
    assert!(!value.is_empty(), "empty B2 template field '{name}'");
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
    fn reviewed_b2_template_bank_is_complete_and_unique() {
        assert_eq!(records().len(), 548);
        let mut ids = HashSet::new();
        for record in records() {
            assert!(ids.insert(record.word_id), "{}", record.word_id);
        }
    }

    #[test]
    fn contains_reviewed_b2_language_and_sense_fixes() {
        let whom = reviewed_b2_template("b2-whom").expect("whom template");
        assert_eq!(whom.1.as_deref(), Some("pron. 谁；什么人"));
        assert_eq!(whom.3, "I know whom you mean.");

        let fundamental =
            reviewed_b2_template("b2-fundamental").expect("fundamental template");
        assert_eq!(fundamental.1.as_deref(), Some("adj. 基本的；根本的"));
        assert_eq!(fundamental.3, "Trust is a fundamental principle.");

        let satisfy = reviewed_b2_template("b2-satisfy").expect("satisfy template");
        assert_eq!(satisfy.3, "The plan should satisfy all the requirements.");

        let slight = reviewed_b2_template("b2-slight").expect("slight template");
        assert_eq!(slight.1.as_deref(), Some("n. 轻视；怠慢"));
        assert_eq!(slight.3, "It was a slight.");

        let cancel = reviewed_b2_template("b2-cancel").expect("cancel template");
        assert_eq!(cancel.3, "We must cancel the meeting.");
    }
}
