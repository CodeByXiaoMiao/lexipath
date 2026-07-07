const ABSTRACT_OR_MASS_NOUNS: &[&str] = &[
    "access", "advice", "advertising", "agriculture", "air", "architecture", "art",
    "attention", "authority", "beauty", "behavior", "bread", "business", "care",
    "charity", "clothing", "coal", "coffee", "communication", "competition",
    "confidence", "construction", "courage", "culture", "data", "democracy",
    "development", "direction", "discussion", "driving", "education", "electricity",
    "employment", "energy", "equipment", "evidence", "experience", "fiction", "fishing",
    "food", "freedom", "fun", "furniture", "growth", "hair", "happiness", "health",
    "history", "homework", "honesty", "hope", "humour", "humor", "importance",
    "independence", "information", "insurance", "intelligence", "knowledge", "language",
    "land", "leadership", "learning", "life", "love", "luck", "luggage", "management",
    "material", "mathematics", "meat", "media", "metal", "milk", "money", "motion",
    "music", "nature", "news", "observation", "oil", "parking", "peace", "permission",
    "physics", "planning", "pleasure", "politics", "pollution", "poverty", "power",
    "practice", "pressure", "progress", "quality", "reality", "reading", "research",
    "respect", "rice", "safety", "sailing", "salt", "science", "security", "singing",
    "skiing", "sleep", "smoking", "society", "space", "strength", "success", "sugar",
    "support", "teaching", "technology", "thinking", "time", "tourism", "traffic",
    "training", "transport", "truth", "washing", "water", "wealth", "weather", "work",
    "working", "writing", "youth",
];

const EVENT_SUBJECT_VERBS: &[&str] = &[
    "appear", "arise", "begin", "break", "change", "continue", "develop", "disappear",
    "emerge", "end", "exist", "fail", "fall", "freeze", "grow", "happen", "improve",
    "increase", "occur", "pass", "remain", "rise", "spread", "start", "stop", "succeed",
];

const PERSON_OBJECT_VERBS: &[&str] = &[
    "accept", "accompany", "admire", "advise", "ask", "call", "convince", "educate",
    "encourage", "forgive", "greet", "help", "inform", "inspire", "invite", "join",
    "lead", "meet", "miss", "persuade", "phone", "promise", "remind", "support",
    "teach", "thank", "trust", "warn",
];

const HUMAN_STATE_ADJECTIVES: &[&str] = &[
    "able", "afraid", "alive", "alone", "ashamed", "asleep", "awake", "aware", "bored",
    "busy", "calm", "certain", "comfortable", "confident", "confused", "conscious",
    "delighted", "disappointed", "eager", "embarrassed", "enthusiastic", "excited",
    "free", "glad", "guilty", "happy", "ill", "interested", "involved", "jealous",
    "lonely", "married", "nervous", "pleased", "proud", "ready", "relaxed", "retired",
    "sad", "satisfied", "shocked", "sick", "sorry", "surprised", "tired", "unable",
    "unemployed", "unhappy", "upset", "willing", "worried",
];

const TIME_PERIOD_NOUNS: &[&str] = &[
    "spring", "summer", "autumn", "fall", "winter", "morning", "afternoon", "evening",
    "night", "weekend",
];

const DAY_ADVERBS: &[&str] = &["today", "tomorrow"];
const PAST_DAY_ADVERBS: &[&str] = &["yesterday"];
const DIRECTION_NOUNS: &[&str] = &["north", "south", "east", "west"];

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
    let meaning_lower = meaning.trim().to_ascii_lowercase();

    if CARDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(&format!("{word} books"), &format!("I have {word} books."), &format!("You have {word} books.")));
    }
    if ORDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(&format!("the {word}"), &format!("This is the {word}."), &format!("It is the {word}.")));
    }
    if DIRECTION_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(&format!("go {word}"), &format!("I go {word}."), &format!("You go {word}.")));
    }
    if DAY_ADVERBS.contains(&lower.as_str()) {
        return Some(tuple(&format!("go {word}"), &format!("I go {word}."), &format!("You come {word}.")));
    }
    if PAST_DAY_ADVERBS.contains(&lower.as_str()) {
        return Some(tuple(&format!("was here {word}"), &format!("I was here {word}."), &format!("He was here {word}.")));
    }
    if TIME_PERIOD_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(&format!("in {word}"), &format!("I go in {word}."), &format!("You come in {word}.")));
    }
    if lower == "based" {
        return Some(tuple("based on this", "It is based on this.", "This is based on it."));
    }
    if lower == "used" {
        return Some(tuple("used this", "I used this.", "You used it."));
    }
    if EVENT_SUBJECT_VERBS.contains(&lower.as_str()) {
        return Some(tuple(&format!("can {word}"), &format!("It can {word}."), &format!("This can {word}.")));
    }
    if PERSON_OBJECT_VERBS.contains(&lower.as_str()) {
        return Some(tuple(&format!("{word} him"), &format!("I can {word} him."), &format!("You can {word} her.")));
    }
    if is_human_state_adjective(&lower, &meaning_lower) {
        return Some(tuple(&format!("am {word}"), &format!("I am {word}."), &format!("You are {word}.")));
    }
    if meaning_lower.starts_with("n.") && (ABSTRACT_OR_MASS_NOUNS.contains(&lower.as_str()) || looks_abstract(&lower)) {
        return Some(tuple(&format!("about {word}"), &format!("This is about {word}."), &format!("I know about {word}.")));
    }
    None
}

fn is_human_state_adjective(word: &str, meaning: &str) -> bool {
    (meaning.starts_with("adj.") || meaning.starts_with("a."))
        && (HUMAN_STATE_ADJECTIVES.contains(&word) || word.ends_with("ed") || word.ends_with("ing"))
}

fn looks_abstract(word: &str) -> bool {
    ["tion", "sion", "ment", "ness", "ity", "ance", "ence", "ship", "ism", "hood", "dom"]
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

    #[test]
    fn human_state_adjectives_use_people() {
        let template = semantic_template("able", "adj. 能够的").expect("template");
        assert_eq!(template.1, "I am able.");
    }

    #[test]
    fn compass_words_use_direction_contexts() {
        let template = semantic_template("south", "n. 南方").expect("template");
        assert_eq!(template.1, "I go south.");
    }

    #[test]
    fn based_receives_required_preposition() {
        let template = semantic_template("based", "v. 以……为基础").expect("template");
        assert_eq!(template.1, "It is based on this.");
    }

    #[test]
    fn yesterday_uses_past_frame() {
        let template = semantic_template("yesterday", "adv. 昨天").expect("template");
        assert_eq!(template.1, "I was here yesterday.");
    }
}
