use std::sync::OnceLock;

const REVIEWED_B1_TEMPLATE_FILES: &[&str] = &[
    include_str!("../assets/example-templates/oxford-b1-01.tsv"),
    include_str!("../assets/example-templates/oxford-b1-02.tsv"),
    include_str!("../assets/example-templates/oxford-b1-03.tsv"),
    include_str!("../assets/example-templates/oxford-b1-04.tsv"),
    include_str!("../assets/example-templates/oxford-b1-05.tsv"),
    include_str!("../assets/example-templates/oxford-b1-06.tsv"),
];

#[derive(Debug)]
struct ReviewedB1Template {
    word_id: &'static str,
    display: Option<&'static str>,
    meaning: Option<&'static str>,
    phrase: &'static str,
    primary_example: &'static str,
    secondary_example: &'static str,
}

static RECORDS: OnceLock<Vec<ReviewedB1Template>> = OnceLock::new();

pub fn reviewed_b1_template(
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

fn records() -> &'static Vec<ReviewedB1Template> {
    RECORDS.get_or_init(|| {
        REVIEWED_B1_TEMPLATE_FILES
            .iter()
            .flat_map(|content| parse_file(content))
            .collect()
    })
}

fn parse_file(content: &'static str) -> Vec<ReviewedB1Template> {
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
            assert!(fields.next().is_none(), "too many B1 template fields");
            ReviewedB1Template {
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
    let value = value.unwrap_or_else(|| panic!("missing B1 template field '{name}'"));
    assert!(!value.is_empty(), "empty B1 template field '{name}'");
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
    fn reviewed_b1_template_bank_is_complete_and_unique() {
        assert_eq!(records().len(), 580);
        let mut ids = HashSet::new();
        for record in records() {
            assert!(ids.insert(record.word_id), "{}", record.word_id);
        }
    }

    #[test]
    fn contains_reviewed_b1_language_and_sense_fixes() {
        let set = reviewed_b1_template("b1-set").expect("set template");
        assert_eq!(set.1.as_deref(), Some("n. 一套；一组"));
        assert_eq!(set.3, "I bought a set of tools.");

        let issue = reviewed_b1_template("b1-issue").expect("issue template");
        assert_eq!(issue.1.as_deref(), Some("n. 问题；议题"));
        assert_eq!(issue.3, "We discussed the issue today.");

        let admit = reviewed_b1_template("b1-admit").expect("admit template");
        assert_eq!(admit.3, "I admit that I was wrong.");

        let queue = reviewed_b1_template("b1-queue").expect("queue template");
        assert_eq!(queue.1.as_deref(), Some("n. 队列"));
        assert_eq!(queue.3, "We joined the queue for tickets.");
    }
}
