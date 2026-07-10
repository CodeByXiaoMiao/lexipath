const ABSTRACT_OR_MASS_NOUNS: &[&str] = &[
    "access", "advice", "advertising", "agriculture", "air", "alcohol", "anger",
    "anxiety", "architecture", "art", "attention", "authority", "beauty", "behavior",
    "bread", "business", "care", "charity", "clothing", "coal", "coffee",
    "communication", "competition", "confidence", "construction", "courage", "culture",
    "damage", "data", "death", "democracy", "destruction", "development", "digestion",
    "direction", "discussion", "driving", "education", "electricity", "employment",
    "energy", "equipment", "evidence", "experience", "fiction", "fishing", "food",
    "freedom", "fun", "furniture", "growth", "happiness", "health", "hearing", "history",
    "homework", "honesty", "hope", "importance", "independence", "information",
    "insurance", "intelligence", "knowledge", "language", "leadership", "learning", "life",
    "love", "luck", "luggage", "management", "material", "mathematics", "meat", "media",
    "metal", "milk", "money", "motion", "music", "nature", "news", "observation",
    "parking", "peace", "permission", "physics", "planning", "pleasure", "politics",
    "pollution", "poverty", "power", "practice", "pressure", "progress", "punishment",
    "quality", "reality", "reading", "research", "respect", "rice", "safety", "sailing",
    "salt", "science", "security", "singing", "skiing", "sleep", "smoking", "society",
    "space", "strength", "success", "sugar", "support", "teaching", "technology",
    "thinking", "time", "tourism", "traffic", "training", "transport", "truth", "violence",
    "washing", "water", "wealth", "weather", "work", "working", "writing", "youth",
];

const EVENT_SUBJECT_VERBS: &[&str] = &[
    "appear", "arise", "begin", "break", "change", "continue", "develop", "disappear",
    "emerge", "end", "exist", "explode", "fail", "fall", "freeze", "grow", "happen",
    "improve", "increase", "occur", "pass", "remain", "rise", "spread", "start", "stop",
    "succeed",
];

const CARDINAL_NUMBERS: &[&str] = &[
    "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    "hundred", "thousand", "million", "billion",
];

const ORDINAL_NUMBERS: &[&str] = &[
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth",
    "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth",
    "sixteenth", "seventeenth", "eighteenth", "nineteenth", "twentieth",
];

pub fn semantic_template(word: &str, meaning: &str) -> Option<(String, String, String)> {
    let lower = word.to_ascii_lowercase();

    if CARDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("{word} books"),
            &format!("I have {word} books."),
            &format!("You have {word} books."),
        ));
    }

    if ORDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("the {word}"),
            &format!("This is the {word}."),
            &format!("It is the {word}."),
        ));
    }

    if EVENT_SUBJECT_VERBS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("can {word}"),
            &format!("It can {word}."),
            &format!("This can {word}."),
        ));
    }

    if meaning.trim().to_ascii_lowercase().starts_with("n.")
        && (ABSTRACT_OR_MASS_NOUNS.contains(&lower.as_str())
            || looks_abstract(&lower))
    {
        return Some(tuple(
            &format!("about {word}"),
            &format!("This is about {word}."),
            &format!("I know about {word}."),
        ));
    }

    None
}

fn looks_abstract(word: &str) -> bool {
    [
        "tion", "sion", "ment", "ness", "ity", "ance", "ence", "ship", "ism",
        "hood", "dom",
    ]
    .iter()
    .any(|suffix| word.ends_with(suffix))
}

fn tuple(phrase: &str, first: &str, second: &str) -> (String, String, String) {
    (phrase.to_owned(), first.to_owned(), second.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_nouns_use_a_topic_context() {
        let template = semantic_template("information", "n. 信息").expect("template");
        assert_eq!(template.1, "This is about information.");
        assert_eq!(template.2, "I know about information.");
    }

    #[test]
    fn ordinal_numbers_require_the() {
        let template = semantic_template("third", "num. 第三").expect("template");
        assert_eq!(template.1, "This is the third.");
    }

    #[test]
    fn event_verbs_use_an_event_subject() {
        let template = semantic_template("happen", "vi. 发生").expect("template");
        assert_eq!(template.1, "It can happen.");
    }
}
