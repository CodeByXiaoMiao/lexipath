#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetClass {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    Pronoun,
    Determiner,
    Modal,
    Number,
    Interjection,
    Other,
}

pub fn normalize_learner_meaning(
    word: &str,
    meaning: &str,
    phrase: &str,
    example: &str,
) -> String {
    let inferred_class = infer_target_class(word, meaning, phrase, example);
    let normalized = meaning.replace("\\n", "\n");
    let lines = normalized
        .lines()
        .map(strip_metadata)
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let class = normalized_single_class(meaning, &lines).unwrap_or(inferred_class);

    let selected = lines
        .iter()
        .find(|line| line_matches_class(line, class))
        .or_else(|| lines.first())
        .map(String::as_str)
        .unwrap_or("词义待补充");
    let selected_body = truncate_secondary_pos(remove_pos_prefix(selected));
    let body = selected_body
        .split(|character| character == ',' || character == '，')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches(|character| matches!(character, '。' | ';' | '；'))
        .trim();
    let body = if body.is_empty() { "词义待补充" } else { body };
    format!("{} {body}", class_label(class, selected))
}

pub fn learner_gloss(meaning: &str) -> &str {
    meaning
        .split_once(' ')
        .map(|(_, gloss)| gloss.trim())
        .filter(|gloss| !gloss.is_empty())
        .unwrap_or(meaning)
}

fn normalized_single_class(original: &str, lines: &[String]) -> Option<TargetClass> {
    let trimmed = original.trim();
    if lines.len() != 1
        || trimmed.contains("\\n")
        || trimmed.contains('\n')
        || trimmed.contains('[')
        || trimmed.contains(']')
        || trimmed.contains(',')
        || trimmed.contains('，')
    {
        return None;
    }
    line_class(&lines[0])
}

fn infer_target_class(word: &str, meaning: &str, phrase: &str, example: &str) -> TargetClass {
    let lower = word.to_ascii_lowercase();
    if let Some(class) = explicit_class(&lower) {
        return class;
    }
    if MODALS.contains(&lower.as_str()) {
        return TargetClass::Modal;
    }
    if DETERMINERS.contains(&lower.as_str()) {
        return TargetClass::Determiner;
    }
    if PRONOUNS.contains(&lower.as_str()) {
        return TargetClass::Pronoun;
    }
    if PREPOSITIONS.contains(&lower.as_str()) {
        return TargetClass::Preposition;
    }
    if CONJUNCTIONS.contains(&lower.as_str()) {
        return TargetClass::Conjunction;
    }

    let phrase_lower = phrase.to_ascii_lowercase();
    let example_lower = example.to_ascii_lowercase();
    if phrase_lower.starts_with("a ")
        || phrase_lower.starts_with("an ")
        || phrase_lower.starts_with("the ")
        || phrase_lower.starts_with("one ")
    {
        return TargetClass::Noun;
    }
    if phrase_lower == lower && first_meaning_class(meaning) == Some(TargetClass::Noun) {
        return TargetClass::Noun;
    }
    if phrase_lower == format!("can {lower}")
        || example_lower.contains(&format!(" can {lower}"))
        || phrase_lower.starts_with(&format!("{lower} "))
            && first_meaning_class(meaning) == Some(TargetClass::Verb)
    {
        return TargetClass::Verb;
    }
    if phrase_lower == format!("is {lower}")
        || phrase_lower == format!("am {lower}")
        || phrase_lower == format!("are {lower}")
        || phrase_lower == format!("be {lower}")
    {
        return TargetClass::Adjective;
    }
    if phrase_lower == format!("do it {lower}")
        || phrase_lower.ends_with(&format!(" {lower}"))
            && first_meaning_class(meaning) == Some(TargetClass::Adverb)
    {
        return TargetClass::Adverb;
    }
    first_meaning_class(meaning).unwrap_or(TargetClass::Other)
}

fn first_meaning_class(meaning: &str) -> Option<TargetClass> {
    meaning
        .replace("\\n", "\n")
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('['))
        .and_then(line_class)
}

fn line_matches_class(line: &str, class: TargetClass) -> bool {
    line_class(line) == Some(class)
        || class == TargetClass::Verb && matches!(line_class(line), Some(TargetClass::Modal))
}

fn line_class(line: &str) -> Option<TargetClass> {
    let lower = line.trim().to_ascii_lowercase();
    if lower.starts_with("n.") || lower.starts_with("pl.") {
        Some(TargetClass::Noun)
    } else if lower.starts_with("v.") || lower.starts_with("vt.") || lower.starts_with("vi.") {
        Some(TargetClass::Verb)
    } else if lower.starts_with("adj.") || lower.starts_with("a.") {
        Some(TargetClass::Adjective)
    } else if lower.starts_with("adv.") || lower.starts_with("r.") {
        Some(TargetClass::Adverb)
    } else if lower.starts_with("prep.") {
        Some(TargetClass::Preposition)
    } else if lower.starts_with("conj.") {
        Some(TargetClass::Conjunction)
    } else if lower.starts_with("pron.") {
        Some(TargetClass::Pronoun)
    } else if lower.starts_with("det.") {
        Some(TargetClass::Determiner)
    } else if lower.starts_with("aux.") || lower.starts_with("modal.") {
        Some(TargetClass::Modal)
    } else if lower.starts_with("num.") {
        Some(TargetClass::Number)
    } else if lower.starts_with("interj.") {
        Some(TargetClass::Interjection)
    } else {
        None
    }
}

fn class_label(class: TargetClass, selected: &str) -> &'static str {
    match class {
        TargetClass::Noun => "n.",
        TargetClass::Verb => "v.",
        TargetClass::Adjective => "adj.",
        TargetClass::Adverb => "adv.",
        TargetClass::Preposition => "prep.",
        TargetClass::Conjunction => "conj.",
        TargetClass::Pronoun => "pron.",
        TargetClass::Determiner => "det.",
        TargetClass::Modal => "modal.",
        TargetClass::Number => "num.",
        TargetClass::Interjection => "interj.",
        TargetClass::Other => match line_class(selected) {
            Some(TargetClass::Noun) => "n.",
            Some(TargetClass::Verb) => "v.",
            Some(TargetClass::Adjective) => "adj.",
            Some(TargetClass::Adverb) => "adv.",
            Some(TargetClass::Preposition) => "prep.",
            Some(TargetClass::Conjunction) => "conj.",
            Some(TargetClass::Pronoun) => "pron.",
            Some(TargetClass::Determiner) => "det.",
            Some(TargetClass::Modal) => "modal.",
            Some(TargetClass::Number) => "num.",
            Some(TargetClass::Interjection) => "interj.",
            _ => "word.",
        },
    }
}

fn explicit_class(word: &str) -> Option<TargetClass> {
    match word {
        "tv" | "internet" | "girlfriend" | "theatre" | "dvd" | "website" | "blog"
        | "cd" | "laptop" | "planning" | "funding" => Some(TargetClass::Noun),
        "bored" | "online" | "long term" => Some(TargetClass::Adjective),
        "significantly" => Some(TargetClass::Adverb),
        "thanks" => Some(TargetClass::Interjection),
        _ => None,
    }
}

fn truncate_secondary_pos(body: &str) -> &str {
    let lower = body.to_ascii_lowercase();
    let mut cut = body.len();
    for marker in [
        "; n.", "；n.", "; v.", "；v.", "; vt.", "；vt.", "; vi.", "；vi.",
        "; adj.", "；adj.", "; a.", "；a.", "; adv.", "；adv.", "; prep.",
        "；prep.", "; conj.", "；conj.", "; pron.", "；pron.", "; aux.", "；aux.",
    ] {
        if let Some(index) = lower.find(marker) {
            cut = cut.min(index);
        }
    }
    body[..cut].trim()
}

fn remove_pos_prefix(line: &str) -> &str {
    let trimmed = line.trim();
    for prefix in [
        "modal.", "prep.", "conj.", "pron.", "det.", "adj.", "adv.", "num.",
        "interj.", "vt.", "vi.", "aux.", "word.", "pl.", "n.", "v.", "a.", "r.",
    ] {
        if trimmed
            .get(..prefix.len())
            .map(|head| head.eq_ignore_ascii_case(prefix))
            .unwrap_or(false)
        {
            return trimmed[prefix.len()..].trim();
        }
    }
    trimmed
}

fn strip_metadata(line: &str) -> String {
    let mut output = String::new();
    let mut bracket_depth = 0usize;
    for character in line.chars() {
        match character {
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            _ if bracket_depth == 0 => output.push(character),
            _ => {}
        }
    }
    output
}

const MODALS: &[&str] = &[
    "can", "cannot", "could", "may", "might", "must", "shall", "should", "will", "would",
];
const DETERMINERS: &[&str] = &[
    "a", "all", "an", "another", "any", "both", "each", "either", "enough", "every",
    "few", "little", "many", "most", "much", "neither", "no", "other", "several", "some",
    "such", "that", "the", "these", "this", "those", "various", "whole",
];
const PRONOUNS: &[&str] = &[
    "anything", "anyone", "anybody", "everybody", "everyone", "everything", "he", "her",
    "hers", "him", "his", "i", "it", "me", "mine", "none", "nothing", "one", "ours", "she",
    "somebody", "someone", "something", "theirs", "them", "they", "us", "we", "what",
    "which", "who", "whom", "whose", "you", "yours",
];
const PREPOSITIONS: &[&str] = &[
    "about", "above", "across", "after", "against", "along", "among", "around", "at", "before",
    "behind", "below", "beneath", "beside", "between", "beyond", "by", "despite", "during",
    "except", "for", "from", "in", "inside", "into", "near", "of", "off", "on", "opposite",
    "outside", "over", "past", "per", "through", "throughout", "till", "to", "toward",
    "towards", "under", "underneath", "unlike", "until", "up", "upon", "with", "within",
    "without",
];
const CONJUNCTIONS: &[&str] = &[
    "although", "and", "as", "because", "but", "if", "nor", "or", "since", "than", "though",
    "unless", "until", "when", "whereas", "whether", "while", "yet",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_one_learner_facing_sense() {
        let result = normalize_learner_meaning(
            "colour",
            "n. 颜色, 面色, 颜料\\nvt. 给……上色, 粉饰",
            "a colour",
            "This is a colour.",
        );
        assert_eq!(result, "n. 颜色");
    }

    #[test]
    fn keeps_curated_part_of_speech_across_repeated_finalization() {
        let first = normalize_learner_meaning(
            "latest",
            "adj. 最新的",
            "the latest one",
            "This is the latest one.",
        );
        let second = normalize_learner_meaning(
            "latest",
            &first,
            "the latest one",
            "This is the latest one.",
        );
        assert_eq!(first, "adj. 最新的");
        assert_eq!(second, first);
    }

    #[test]
    fn chooses_the_taught_modal_sense() {
        let result = normalize_learner_meaning(
            "will",
            "n. 意志, 决心\\naux. 将, 愿意",
            "will go",
            "I will go.",
        );
        assert_eq!(result, "modal. 将");
    }
}
