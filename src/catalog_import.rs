use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use crate::course::{
    CoursePack, Lesson, Question, Reading, SentenceItem, Stage, WordItem,
};
use crate::embedded_course;
use crate::validator::validate_course;

const LEVELS: [&str; 4] = ["A1", "A2", "B1", "B2"];
const UNIT_SIZE: usize = 6;

#[derive(Debug)]
struct ImportArguments {
    ogden_list: PathBuf,
    word_list: PathBuf,
    dictionary: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone)]
struct DictionaryEntry {
    word: String,
    phonetic: String,
    translation: String,
    part_of_speech: String,
    rank: u32,
}

pub fn import_catalog(arguments: &[String]) -> anyhow::Result<()> {
    let arguments = parse_arguments(arguments)?;
    let ogden_words = load_line_word_list(&arguments.ogden_list)?;
    let levels = load_word_levels(&arguments.word_list)?;

    let mut target_words = collect_target_words(&levels);
    target_words.extend(ogden_words.iter().map(|word| normalize_entry(word)));
    let dictionary = load_dictionary(&arguments.dictionary, &target_words)?;

    let mut course = embedded_course::load()?;
    let mut already_present = course_word_keys(&course);

    let ogden_entries = resolve_entries(
        "Ogden 850",
        &ogden_words,
        &dictionary,
        &already_present,
    )?;
    already_present.extend(
        ogden_entries
            .iter()
            .map(|entry| normalize_entry(&entry.word)),
    );
    course.stages.push(build_stage(
        "ogden-850",
        "Ogden Basic English 850",
        "ogden",
        &ogden_entries,
    ));

    for level in LEVELS {
        let words = levels
            .get(level)
            .with_context(|| format!("word list is missing CEFR level {level}"))?;
        let entries = resolve_entries(level, words, &dictionary, &already_present)?;
        already_present.extend(entries.iter().map(|entry| normalize_entry(&entry.word)));
        course.stages.push(build_stage(
            &format!("oxford-{}", level.to_ascii_lowercase()),
            &format!("Oxford 3000 {level}"),
            &level.to_ascii_lowercase(),
            &entries,
        ));
    }

    course.id = "lexipath-fixed-path".to_owned();
    course.title = "LexiPath 固定英语学习计划".to_owned();
    course.version = course.version.saturating_add(1);

    if let Err(errors) = validate_course(&course) {
        let details = errors
            .into_iter()
            .take(40)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        bail!("generated catalog failed validation:\n{details}");
    }

    if let Some(parent) = arguments.output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = arguments.output.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec(&course)?)?;
    fs::rename(&temporary, &arguments.output)?;

    println!(
        "generated {} stages, {} units and {} unique entries at {}",
        course.stages.len(),
        course
            .stages
            .iter()
            .map(|stage| stage.lessons.len())
            .sum::<usize>(),
        course
            .stages
            .iter()
            .flat_map(|stage| stage.lessons.iter())
            .map(|lesson| lesson.new_words.len())
            .sum::<usize>(),
        arguments.output.display()
    );
    Ok(())
}

fn parse_arguments(arguments: &[String]) -> anyhow::Result<ImportArguments> {
    let mut ogden_list = None;
    let mut word_list = None;
    let mut dictionary = None;
    let mut output = None;
    let mut index = 0;

    while index < arguments.len() {
        let value = arguments
            .get(index + 1)
            .with_context(|| format!("missing value after {}", arguments[index]))?;
        match arguments[index].as_str() {
            "--ogden" => ogden_list = Some(PathBuf::from(value)),
            "--word-list" => word_list = Some(PathBuf::from(value)),
            "--dictionary" => dictionary = Some(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            other => bail!("unknown import option: {other}"),
        }
        index += 2;
    }

    Ok(ImportArguments {
        ogden_list: ogden_list.context("--ogden is required")?,
        word_list: word_list.context("--word-list is required")?,
        dictionary: dictionary.context("--dictionary is required")?,
        output: output.context("--output is required")?,
    })
}

fn load_line_word_list(path: &Path) -> anyhow::Result<Vec<String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read word list {}", path.display()))?;
    let mut seen = HashSet::new();
    let mut words = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        for variant in line.split('/') {
            let word = variant.trim();
            let normalized = normalize_entry(word);
            if !normalized.is_empty() && seen.insert(normalized) {
                words.push(word.to_owned());
            }
        }
    }
    Ok(words)
}

fn load_word_levels(path: &Path) -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read word list {}", path.display()))?;
    let mut levels: BTreeMap<String, Vec<String>> =
        serde_json::from_str(&text).context("failed to parse CEFR word list")?;
    deduplicate_levels(&mut levels);
    Ok(levels)
}

fn deduplicate_levels(levels: &mut BTreeMap<String, Vec<String>>) {
    let mut seen = HashSet::<String>::new();
    for level in LEVELS {
        if let Some(words) = levels.get_mut(level) {
            words.retain(|word| {
                let normalized = normalize_entry(word);
                !normalized.is_empty() && seen.insert(normalized)
            });
        }
    }
}

fn collect_target_words(levels: &BTreeMap<String, Vec<String>>) -> HashSet<String> {
    LEVELS
        .iter()
        .filter_map(|level| levels.get(*level))
        .flatten()
        .map(|word| normalize_entry(word))
        .collect()
}

fn load_dictionary(
    path: &Path,
    target_words: &HashSet<String>,
) -> anyhow::Result<HashMap<String, DictionaryEntry>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("failed to open dictionary {}", path.display()))?;
    let headers = reader.headers()?.clone();
    let word_index = header_index(&headers, "word")?;
    let phonetic_index = header_index(&headers, "phonetic")?;
    let translation_index = header_index(&headers, "translation")?;
    let pos_index = header_index(&headers, "pos")?;
    let bnc_index = header_index(&headers, "bnc")?;
    let frequency_index = header_index(&headers, "frq")?;

    let mut output = HashMap::new();
    for row in reader.records() {
        let row = row?;
        let word = row.get(word_index).unwrap_or_default().trim();
        let key = normalize_entry(word);
        if !target_words.contains(&key) {
            continue;
        }
        let translation = first_translation(row.get(translation_index).unwrap_or_default());
        let phonetic = row.get(phonetic_index).unwrap_or_default().trim();
        if translation.is_empty() || phonetic.is_empty() {
            continue;
        }
        let bnc = parse_rank(row.get(bnc_index).unwrap_or_default());
        let frequency = parse_rank(row.get(frequency_index).unwrap_or_default());
        let rank = bnc.min(frequency).min(900_000);
        output.entry(key).or_insert_with(|| DictionaryEntry {
            word: word.to_owned(),
            phonetic: phonetic.to_owned(),
            translation,
            part_of_speech: row.get(pos_index).unwrap_or_default().to_owned(),
            rank,
        });
    }
    Ok(output)
}

fn resolve_entries(
    label: &str,
    words: &[String],
    dictionary: &HashMap<String, DictionaryEntry>,
    already_present: &HashSet<String>,
) -> anyhow::Result<Vec<DictionaryEntry>> {
    let expected = words
        .iter()
        .map(|word| normalize_entry(word))
        .filter(|word| !already_present.contains(word))
        .collect::<HashSet<_>>();
    let mut entries = expected
        .iter()
        .filter_map(|word| dictionary.get(word).cloned())
        .collect::<Vec<_>>();
    let found = entries
        .iter()
        .map(|entry| normalize_entry(&entry.word))
        .collect::<HashSet<_>>();
    let mut missing = expected.difference(&found).cloned().collect::<Vec<_>>();
    missing.sort();
    if !missing.is_empty() {
        bail!(
            "dictionary is missing {} required {label} entries; first items: {}",
            missing.len(),
            missing.into_iter().take(20).collect::<Vec<_>>().join(", ")
        );
    }
    entries.sort_by_key(|entry| entry.rank);
    Ok(entries)
}

fn header_index(headers: &csv::StringRecord, name: &str) -> anyhow::Result<usize> {
    headers
        .iter()
        .position(|header| header.eq_ignore_ascii_case(name))
        .with_context(|| format!("dictionary has no '{name}' column"))
}

fn first_translation(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .chars()
        .take(90)
        .collect()
}

fn parse_rank(value: &str) -> u32 {
    value.trim().parse::<u32>().unwrap_or(900_000)
}

fn course_word_keys(course: &CoursePack) -> HashSet<String> {
    course
        .stages
        .iter()
        .flat_map(|stage| stage.lessons.iter())
        .flat_map(|lesson| lesson.new_words.iter())
        .map(|word| normalize_entry(&word.text))
        .collect()
}

fn build_stage(id: &str, title: &str, unit_prefix: &str, entries: &[DictionaryEntry]) -> Stage {
    let lessons = entries
        .chunks(UNIT_SIZE)
        .enumerate()
        .map(|(index, words)| build_lesson(unit_prefix, index, words))
        .collect();
    Stage {
        id: id.to_owned(),
        title: title.to_owned(),
        lessons,
    }
}

fn build_lesson(prefix: &str, unit_index: usize, entries: &[DictionaryEntry]) -> Lesson {
    let day = unit_index / 2 + 1;
    let slot = if unit_index % 2 == 0 { "A" } else { "B" };
    let mut word_items = Vec::new();
    let mut reading_sentences = Vec::new();
    let mut questions = Vec::new();

    for entry in entries {
        let (phrase, first_sentence, second_sentence) = templates(entry);
        reading_sentences.push(first_sentence.clone());
        reading_sentences.push(second_sentence);
        word_items.push(WordItem {
            id: format!("{}-{}", prefix, slug(&entry.word)),
            text: entry.word.clone(),
            ipa: format!("/{}/", entry.phonetic.trim_matches('/')),
            meaning: entry.translation.clone(),
            phrase,
            example: first_sentence,
        });
        questions.push(Question {
            prompt: format!("选择包含“{}”对应词义的英文句子。", entry.translation),
            options: Vec::new(),
            correct_index: 0,
        });
    }

    let option_sentences = reading_sentences
        .iter()
        .step_by(2)
        .cloned()
        .collect::<Vec<_>>();
    for (index, question) in questions.iter_mut().enumerate() {
        question.options = option_sentences.clone();
        question.correct_index = index;
    }

    let sentences = word_items
        .iter()
        .take(2)
        .enumerate()
        .map(|(index, word)| SentenceItem {
            text: word.example.clone(),
            meaning: format!("练习句 {}：{}", index + 1, word.meaning),
        })
        .collect();

    Lesson {
        id: format!("{}-unit-{:03}", prefix, unit_index + 1),
        title: format!("第 {day} 天 · 单元 {slot}"),
        new_words: word_items,
        sentences,
        reading: Reading {
            title: "配套零生词阅读".to_owned(),
            sentences: reading_sentences,
            questions,
        },
    }
}

fn templates(entry: &DictionaryEntry) -> (String, String, String) {
    let word = entry.word.trim();
    if let Some(template) = function_word_template(word) {
        return template;
    }

    match primary_pos(&entry.part_of_speech) {
        'n' => (
            format!("a {word}"),
            format!("This is a {word}."),
            format!("It is a {word}."),
        ),
        'v' => (
            format!("can {word}"),
            format!("I can {word}."),
            format!("You can {word}."),
        ),
        'a' | 'j' => (
            format!("is {word}"),
            format!("It is {word}."),
            format!("This is {word}."),
        ),
        'r' => (
            format!("{word} now"),
            format!("It is {word} now."),
            format!("This is {word} now."),
        ),
        _ => (
            word.to_owned(),
            format!("This is {word}."),
            format!("It is {word}."),
        ),
    }
}

fn function_word_template(word: &str) -> Option<(String, String, String)> {
    let normalized = word.to_ascii_lowercase();
    let sentences = match normalized.as_str() {
        "the" => ("the book", "This is the book.", "The book is here."),
        "an" => ("an open book", "This is an open book.", "It is an open book."),
        "to" => ("go to", "I go to you.", "You come to me."),
        "from" => ("come from", "I come from there.", "You come from here."),
        "with" => ("with you", "I am with you.", "You are with me."),
        "by" => ("by you", "I am by you.", "You are by me."),
        "of" => ("one of two", "This is one of two.", "It is one of three."),
        "for" => ("for you", "This is for you.", "It is for me."),
        "up" => ("go up", "I can go up.", "You can go up."),
        "down" => ("go down", "I can go down.", "You can go down."),
        "off" => ("go off", "I can go off.", "You can go off."),
        "over" => ("go over", "I can go over.", "You can go over."),
        "through" => ("go through", "I can go through.", "You can go through."),
        "after" => ("after this", "I can go after this.", "You can come after this."),
        "before" => ("before this", "I can go before this.", "You can come before this."),
        "again" => ("go again", "I can go again.", "You can come again."),
        "away" => ("go away", "I can go away.", "You can go away."),
        _ => return None,
    };
    Some((sentences.0.to_owned(), sentences.1.to_owned(), sentences.2.to_owned()))
}

fn primary_pos(value: &str) -> char {
    value
        .split('/')
        .next()
        .and_then(|item| item.trim().chars().next())
        .unwrap_or('x')
        .to_ascii_lowercase()
}

fn normalize_entry(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphabetic() && character != '\'')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn slug(value: &str) -> String {
    normalize_entry(value).replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_spelling_is_kept_at_the_earliest_level_only() {
        let mut levels = BTreeMap::from([
            ("A1".to_owned(), vec!["adult".to_owned(), "book".to_owned()]),
            ("A2".to_owned(), vec!["adult".to_owned(), "bridge".to_owned()]),
            ("B1".to_owned(), vec!["bridge".to_owned(), "career".to_owned()]),
            ("B2".to_owned(), vec!["career".to_owned(), "debate".to_owned()]),
        ]);
        deduplicate_levels(&mut levels);
        assert_eq!(levels["A1"], ["adult", "book"]);
        assert_eq!(levels["A2"], ["bridge"]);
        assert_eq!(levels["B1"], ["career"]);
        assert_eq!(levels["B2"], ["debate"]);
    }

    #[test]
    fn slash_variants_become_separate_ogden_entries() {
        let path = std::env::temp_dir().join("lexipath-ogden-test.txt");
        fs::write(&path, "grey/gray\nbook\n").expect("write test list");
        let words = load_line_word_list(&path).expect("load test list");
        let _ = fs::remove_file(path);
        assert_eq!(words, ["grey", "gray", "book"]);
    }

    #[test]
    fn hyphenated_entries_use_a_stable_dictionary_key() {
        assert_eq!(normalize_entry("T-shirt"), "t shirt");
        assert_eq!(normalize_entry("T shirt"), "t shirt");
    }
}
