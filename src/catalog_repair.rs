use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;

#[derive(Debug)]
struct Arguments {
    ogden: PathBuf,
    word_list: PathBuf,
    input: PathBuf,
    output: PathBuf,
}

pub fn repair(arguments: &[String]) -> anyhow::Result<()> {
    let arguments = parse_arguments(arguments)?;
    let targets = load_targets(&arguments.ogden, &arguments.word_list)?;

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&arguments.input)?;
    let headers = reader.headers()?.clone();
    let word_index = header_index(&headers, "word")?;
    let phonetic_index = header_index(&headers, "phonetic")?;
    let translation_index = header_index(&headers, "translation")?;

    let mut phonetics = HashMap::<String, String>::new();
    let mut target_rows = HashMap::<String, csv::StringRecord>::new();
    let mut complete_targets = HashSet::<String>::new();

    for row in reader.records() {
        let row = row?;
        let key = normalize(row.get(word_index).unwrap_or_default());
        if key.is_empty() {
            continue;
        }
        let phonetic = row.get(phonetic_index).unwrap_or_default().trim();
        if !phonetic.is_empty() {
            phonetics.entry(key.clone()).or_insert_with(|| phonetic.to_owned());
        }
        if targets.contains(&key) {
            let translation = row.get(translation_index).unwrap_or_default().trim();
            if !translation.is_empty() {
                target_rows.entry(key.clone()).or_insert_with(|| row.clone());
            }
            if !phonetic.is_empty() && !translation.is_empty() {
                complete_targets.insert(key);
            }
        }
    }

    let mut writer = csv::Writer::from_path(&arguments.output)?;
    writer.write_record(&headers)?;
    let mut repaired = 0usize;
    let mut unresolved = Vec::new();

    for target in targets {
        if complete_targets.contains(&target) {
            continue;
        }
        let Some(row) = target_rows.get(&target) else {
            unresolved.push(target);
            continue;
        };
        let Some(phonetic) = derive_phonetic(&target, &phonetics) else {
            unresolved.push(target);
            continue;
        };
        let mut fields = row.iter().map(str::to_owned).collect::<Vec<_>>();
        fields[phonetic_index] = phonetic;
        writer.write_record(fields)?;
        repaired += 1;
    }
    writer.flush()?;
    unresolved.sort();

    println!("repaired {repaired} dictionary entries");
    if !unresolved.is_empty() {
        println!(
            "unresolved entries remain for later explicit review: {}",
            unresolved.into_iter().take(40).collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

fn derive_phonetic(word: &str, phonetics: &HashMap<String, String>) -> Option<String> {
    if word.contains(' ') {
        return word
            .split_whitespace()
            .map(|token| phonetics.get(token).cloned())
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join(" "));
    }

    for (base, suffix) in inflection_candidates(word) {
        if let Some(phonetic) = phonetics.get(&base) {
            return Some(format!("{}{}", phonetic.trim_end_matches('/'), suffix));
        }
    }

    for split in 2..word.len().saturating_sub(1) {
        if !word.is_char_boundary(split) {
            continue;
        }
        let left = &word[..split];
        let right = &word[split..];
        if let (Some(left_ipa), Some(right_ipa)) = (phonetics.get(left), phonetics.get(right)) {
            return Some(format!("{} {}", left_ipa.trim_matches('/'), right_ipa.trim_matches('/')));
        }
    }
    None
}

fn inflection_candidates(word: &str) -> Vec<(String, &'static str)> {
    let mut output = Vec::new();
    if let Some(stem) = word.strip_suffix("ied") {
        output.push((format!("{stem}y"), "d"));
    }
    if let Some(stem) = word.strip_suffix("ed") {
        output.push((stem.to_owned(), "d"));
        output.push((format!("{stem}e"), "d"));
    }
    if let Some(stem) = word.strip_suffix("ing") {
        output.push((stem.to_owned(), "ɪŋ"));
        output.push((format!("{stem}e"), "ɪŋ"));
    }
    if let Some(stem) = word.strip_suffix("ies") {
        output.push((format!("{stem}y"), "z"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        output.push((stem.to_owned(), "ɪz"));
    }
    if let Some(stem) = word.strip_suffix('s') {
        output.push((stem.to_owned(), "z"));
    }
    output
}

fn load_targets(ogden: &Path, word_list: &Path) -> anyhow::Result<HashSet<String>> {
    let mut output = HashSet::new();
    for line in fs::read_to_string(ogden)?.lines() {
        for variant in line.trim().split('/') {
            let key = normalize(variant);
            if !key.is_empty() {
                output.insert(key);
            }
        }
    }

    let levels: BTreeMap<String, Vec<String>> =
        serde_json::from_str(&fs::read_to_string(word_list)?)?;
    for words in levels.values() {
        for word in words {
            let key = normalize(word);
            if !key.is_empty() {
                output.insert(key);
            }
        }
    }
    Ok(output)
}

fn parse_arguments(arguments: &[String]) -> anyhow::Result<Arguments> {
    let mut values = HashMap::<String, PathBuf>::new();
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .with_context(|| format!("missing value after {}", arguments[index]))?;
        values.insert(arguments[index].clone(), PathBuf::from(value));
        index += 2;
    }
    Ok(Arguments {
        ogden: values.remove("--ogden").context("--ogden is required")?,
        word_list: values
            .remove("--word-list")
            .context("--word-list is required")?,
        input: values.remove("--input").context("--input is required")?,
        output: values.remove("--output").context("--output is required")?,
    })
}

fn header_index(headers: &csv::StringRecord, name: &str) -> anyhow::Result<usize> {
    headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(name))
        .with_context(|| format!("dictionary has no '{name}' column"))
}

fn normalize(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphabetic() && character != '\'')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inflection_candidates_cover_common_forms() {
        assert!(inflection_candidates("including")
            .iter()
            .any(|(base, suffix)| base == "include" && *suffix == "ɪŋ"));
        assert!(inflection_candidates("divorced")
            .iter()
            .any(|(base, suffix)| base == "divorce" && *suffix == "d"));
    }

    #[test]
    fn compound_phonetic_uses_known_parts() {
        let phonetics = HashMap::from([
            ("smart".to_owned(), "smɑːrt".to_owned()),
            ("phone".to_owned(), "foʊn".to_owned()),
        ]);
        assert_eq!(
            derive_phonetic("smartphone", &phonetics),
            Some("smɑːrt foʊn".to_owned())
        );
    }
}
