const ABSTRACT_OR_MASS_NOUNS: &[&str] = &[
    "access", "advice", "advertising", "agriculture", "air", "architecture", "art",
    "attention", "authority", "beauty", "behavior", "behaviour", "bread", "business",
    "care", "cash", "charity", "clothing", "coal", "coffee", "communication",
    "confidence", "construction", "courage", "culture", "data", "democracy",
    "development", "direction", "driving", "education", "electricity", "employment",
    "energy", "equipment", "evidence", "experience", "fiction", "fishing", "food",
    "freedom", "fun", "furniture", "growth", "hair", "happiness", "health",
    "homework", "honesty", "hope", "humour", "humor", "importance", "independence",
    "information", "insurance", "intelligence", "knowledge", "language", "land",
    "leadership", "learning", "life", "love", "luck", "luggage", "management",
    "mathematics", "meat", "media", "metal", "milk", "money", "motion", "music",
    "nature", "news", "oil", "parking", "peace", "permission", "physics",
    "planning", "pleasure", "politics", "pollution", "poverty", "power",
    "practice", "pressure", "progress", "reading", "research", "respect", "rice",
    "rubbish", "safety", "sailing", "salt", "science", "security", "singing",
    "skiing", "sleep", "smoking", "society", "space", "strength", "success",
    "sugar", "support", "teaching", "technology", "thinking", "time", "tourism",
    "traffic", "training", "transport", "truth", "washing", "water", "wealth",
    "weather", "work", "working", "writing", "youth",
];

const EVENT_SUBJECT_VERBS: &[&str] = &[
    "appear", "begin", "break", "change", "continue", "develop", "disappear",
    "emerge", "end", "exist", "fail", "fall", "freeze", "grow", "happen",
    "improve", "increase", "pass", "remain", "rise", "spread", "start", "stop",
    "succeed",
];

const PERSON_OBJECT_VERBS: &[&str] = &[
    "accept", "accompany", "admire", "advise", "ask", "call", "convince", "educate",
    "encourage", "forgive", "greet", "help", "inform", "inspire", "invite", "join",
    "lead", "meet", "miss", "persuade", "phone", "promise", "remind", "support",
    "teach", "thank", "trust", "warn",
];

const HUMAN_STATE_ADJECTIVES: &[&str] = &[
    "able", "afraid", "alive", "alone", "ashamed", "asleep", "awake", "aware",
    "bored", "busy", "calm", "comfortable", "confident", "confused", "conscious",
    "delighted", "disappointed", "eager", "embarrassed", "enthusiastic", "excited",
    "free", "glad", "guilty", "happy", "ill", "interested", "jealous", "lonely",
    "married", "nervous", "pleased", "proud", "ready", "relaxed", "retired", "sad",
    "satisfied", "shocked", "sick", "sorry", "surprised", "tired", "unable",
    "unemployed", "unhappy", "upset", "willing", "worried",
];

const TIME_PERIOD_NOUNS: &[&str] = &[
    "spring", "summer", "autumn", "fall", "winter", "morning", "afternoon", "evening",
    "night",
];

const DAY_ADVERBS: &[&str] = &["today", "tomorrow"];
const PAST_DAY_ADVERBS: &[&str] = &["yesterday"];
const DIRECTION_NOUNS: &[&str] = &["north", "south", "east", "west"];

const CARDINAL_NUMBERS: &[&str] = &[
    "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
    "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

const LARGE_NUMBER_NOUNS: &[&str] = &["hundred", "thousand", "million", "billion"];

const ORDINAL_NUMBERS: &[&str] = &[
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth",
    "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth",
    "sixteenth", "seventeenth", "eighteenth", "nineteenth", "twentieth",
];

pub fn semantic_template(word: &str, meaning: &str) -> Option<(String, String, String)> {
    let lower = word.to_ascii_lowercase();
    let meaning_lower = meaning.trim().to_ascii_lowercase();

    if let Some(template) = fixed_semantic_template(word, &lower) {
        return Some(template);
    }
    if CARDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("{word} books"),
            &format!("I have {word} books."),
            &format!("You have {word} books."),
        ));
    }
    if LARGE_NUMBER_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("one {word}"),
            &format!("This is one {word}."),
            &format!("I see one {word}."),
        ));
    }
    if ORDINAL_NUMBERS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("the {word}"),
            &format!("This is the {word}."),
            &format!("It is the {word}."),
        ));
    }
    if DIRECTION_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("go {word}"),
            &format!("I go {word}."),
            &format!("You go {word}."),
        ));
    }
    if DAY_ADVERBS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("go {word}"),
            &format!("I go {word}."),
            &format!("You come {word}."),
        ));
    }
    if PAST_DAY_ADVERBS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("was here {word}"),
            &format!("I was here {word}."),
            &format!("He was here {word}."),
        ));
    }
    if TIME_PERIOD_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("in the {word}"),
            &format!("I go in the {word}."),
            &format!("You come in the {word}."),
        ));
    }
    if lower == "based" && !is_noun_meaning(&meaning_lower) {
        return Some(tuple(
            "based on this",
            "It is based on this.",
            "This is based on it.",
        ));
    }
    if EVENT_SUBJECT_VERBS.contains(&lower.as_str()) && is_verb_meaning(&meaning_lower) {
        return Some(tuple(
            &format!("can {word}"),
            &format!("This can {word}."),
            &format!("It can {word}."),
        ));
    }
    if PERSON_OBJECT_VERBS.contains(&lower.as_str()) && is_verb_meaning(&meaning_lower) {
        return Some(tuple(
            &format!("{word} him"),
            &format!("I can {word} him."),
            &format!("You can {word} her."),
        ));
    }
    if is_human_state_adjective(&lower, &meaning_lower) {
        return Some(tuple(
            &format!("am {word}"),
            &format!("I am {word}."),
            &format!("You are {word}."),
        ));
    }
    if is_noun_meaning(&meaning_lower) && ABSTRACT_OR_MASS_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            &format!("know about {word}"),
            &format!("I know about {word}."),
            &format!("You know about {word}."),
        ));
    }
    None
}

fn fixed_semantic_template(word: &str, lower: &str) -> Option<(String, String, String)> {
    let values = match lower {
        "belong" => ("belong to me", "It belongs to me.", "This belongs to you."),
        "occur" => ("can occur", "A change can occur.", "This can occur."),
        "arise" => ("can arise", "A change can arise.", "This can arise."),
        "used" => ("a used book", "This is a used book.", "I have a used book."),
        "prefer" => ("prefer this", "I prefer this.", "You prefer it."),
        "deserve" => ("deserve this", "I deserve this.", "You deserve it."),
        "think" => ("think about this", "I can think about this.", "You can think about it."),
        "spend" => ("spend money", "I spend money.", "You spend money."),
        "serve" => ("serve food", "I serve food.", "You serve food."),
        "involve" => ("involves this", "This involves work.", "It involves time."),
        "realize" => ("realize this", "I realize this.", "You realize it."),
        "affect" => ("affect me", "This can affect me.", "It can affect you."),
        "except" => ("except Monday", "I go every day except Monday.", "You come every day except Monday."),
        "relate" => ("relate to this", "This relates to me.", "It relates to you."),
        "reflect" => ("reflect light", "It can reflect light.", "This can reflect light."),
        "admit" => ("admit this", "I admit this.", "You admit it."),
        "lay" => ("lay it down", "I can lay it down.", "You can lay it down."),
        "obtain" => ("obtain a result", "I can obtain a result.", "You can obtain a result."),
        "propose" => ("propose a plan", "I propose a plan.", "You propose a plan."),
        "contribute" => ("contribute to this", "I contribute to this.", "You contribute to it."),
        "surround" => ("surround this", "This can surround it.", "It can surround this."),
        "found" => ("found a school", "I can found a school.", "You can found a school."),
        "warm" => ("warm today", "It is warm today.", "This is warm today."),
        "best" => ("the best", "It is the best.", "This is the best."),
        "maybe" => ("maybe the same", "Maybe it is the same.", "Maybe this is the same."),
        "else" => ("something else", "This is something else.", "I see something else."),
        "further" => ("further away", "It is further away.", "This is further away."),
        "lucky" => ("am lucky", "He is lucky.", "She is lucky."),
        "teenage" => ("a teenage boy", "He is a teenage boy.", "She is a teenage girl."),
        "religious" => ("a religious person", "He is a religious person.", "She is a religious person."),
        "unlike" => ("unlike this", "It is unlike this.", "This is unlike that."),
        "grateful" => ("am grateful", "I am grateful.", "You are grateful."),
        "outdoor" => ("an outdoor game", "This is an outdoor game.", "It is an outdoor game."),
        "drunk" => ("is drunk", "He is drunk.", "She is drunk."),
        "former" => ("the former one", "This is the former one.", "It is the former one."),
        "civil" => ("civil law", "This is civil law.", "It is civil law."),
        "actual" => ("the actual result", "This is the actual result.", "It is the actual result."),
        "overall" => ("overall", "Overall, it is good.", "Overall, this is good."),
        "corporate" => ("corporate work", "This is corporate work.", "It is corporate work."),
        "urban" => ("an urban area", "This is an urban area.", "It is an urban area."),
        "upper" => ("the upper part", "This is the upper part.", "It is the upper part."),
        "inner" => ("the inner part", "This is the inner part.", "It is the inner part."),
        "numerous" => ("numerous books", "There are numerous books.", "I see numerous books."),
        "pregnant" => ("is pregnant", "She is pregnant.", "The woman is pregnant."),
        "deliberate" => ("a deliberate action", "This is a deliberate action.", "It is a deliberate action."),
        "festival" => ("a festival", "It is a festival.", "This is a festival."),
        "weekend" => ("on the weekend", "I go on the weekend.", "You come on the weekend."),
        _ => return None,
    };
    Some(tuple(values.0, values.1, values.2))
}

fn is_human_state_adjective(word: &str, meaning: &str) -> bool {
    (meaning.starts_with("adj.") || meaning.starts_with("a."))
        && HUMAN_STATE_ADJECTIVES.contains(&word)
}

fn is_noun_meaning(meaning: &str) -> bool {
    meaning.starts_with("n.")
}

fn is_verb_meaning(meaning: &str) -> bool {
    meaning.starts_with("v.") || meaning.starts_with("vt.") || meaning.starts_with("vi.")
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
        assert_eq!(template.1, "I know about information.");
        assert_eq!(template.2, "You know about information.");
    }

    #[test]
    fn suffix_only_count_nouns_do_not_get_bare_about_frames() {
        assert!(semantic_template("question", "n. 问题").is_none());
    }

    #[test]
    fn ordinal_numbers_require_the() {
        let template = semantic_template("third", "num. 第三").expect("template");
        assert_eq!(template.1, "This is the third.");
    }

    #[test]
    fn large_numbers_use_one_before_the_number_word() {
        let template = semantic_template("million", "num. 百万").expect("template");
        assert_eq!(template.1, "This is one million.");
    }

    #[test]
    fn event_verbs_use_an_event_subject() {
        let template = semantic_template("happen", "vi. 发生").expect("template");
        assert_eq!(template.1, "This can happen.");
    }

    #[test]
    fn human_state_adjectives_use_people() {
        let template = semantic_template("able", "adj. 能够的").expect("template");
        assert_eq!(template.1, "I am able.");
    }

    #[test]
    fn ing_adjectives_are_not_automatically_people_states() {
        assert!(semantic_template("interesting", "adj. 有趣的").is_none());
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

    #[test]
    fn seasons_use_the_in_controlled_frames() {
        let template = semantic_template("summer", "n. 夏天").expect("template");
        assert_eq!(template.1, "I go in the summer.");
    }
}
