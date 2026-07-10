const MANNER_ADVERBS: &[&str] = &[
    "badly", "carefully", "clearly", "closely", "correctly", "deliberately",
    "differently", "directly", "easily", "effectively", "happily", "heavily",
    "loudly", "naturally", "normally", "perfectly", "personally", "properly",
    "quickly", "quietly", "rapidly", "sadly", "seriously", "slowly", "strongly",
    "successfully",
];

const FREQUENCY_ADVERBS: &[&str] = &[
    "always", "commonly", "constantly", "frequently", "generally", "never",
    "occasionally", "often", "rarely", "regularly", "sometimes", "usually",
];

const DIRECTION_ADVERBS: &[&str] = &[
    "abroad", "anywhere", "away", "backwards", "downwards", "elsewhere",
    "everywhere", "indoors", "later", "nowhere", "somewhere", "soon", "upwards",
];

const SAME_DEGREE_ADVERBS: &[&str] = &[
    "almost", "basically", "completely", "entirely", "exactly", "fairly", "largely",
    "mostly", "nearly", "partly", "quite", "really", "relatively", "totally", "truly",
];

const PROBABILITY_ADVERBS: &[&str] = &[
    "apparently", "certainly", "definitely", "perhaps", "possibly", "probably",
    "surely",
];

const BIG_DEGREE_ADVERBS: &[&str] = &[
    "absolutely", "especially", "extremely", "increasingly", "incredibly",
    "particularly", "slightly", "too",
];

const SEQUENCE_ADVERBS: &[&str] = &[
    "afterwards", "eventually", "finally", "first", "firstly", "secondly",
];

const PERSON_STATE_ADJECTIVES: &[&str] = &[
    "able", "angry", "annoyed", "amazed", "armed", "busy", "calm", "careful",
    "careless", "concerned", "confident", "conscious", "convinced", "dead",
    "delighted", "determined", "disappointed", "divorced", "dressed", "educated",
    "embarrassed", "engaged", "excited", "experienced", "familiar", "frightened",
    "friendly", "guilty", "happy", "honest", "impressed", "injured", "innocent",
    "interested", "lonely", "married", "nervous", "pleased", "polite", "poor",
    "prepared", "proud", "qualified", "relaxed", "responsible", "retired", "rich",
    "rude", "sad", "satisfied", "scared", "serious", "shocked", "single",
    "surprised", "talented", "tired", "unconscious", "unemployed", "unhappy",
    "worried", "young",
];

const COMMON_COUNT_NOUNS: &[&str] = &[
    "blog", "girlfriend", "laptop", "theatre", "website",
];

pub fn normalize_display(word: &str) -> String {
    match word {
        "CORE" => "core".to_owned(),
        "Conservative" => "conservative".to_owned(),
        "North" => "north".to_owned(),
        "God" => "god".to_owned(),
        "Polish" => "polish".to_owned(),
        _ => word.to_owned(),
    }
}

pub fn reviewed_template(word: &str) -> Option<(String, String, String)> {
    let lower = word.to_ascii_lowercase();

    let fixed = match lower.as_str() {
        // High-risk generated frames found during manual whole-course review.
        "up" => ("go up", "I go up.", "You go up."),
        "down" => ("go down", "I go down.", "You go down."),
        "back" => ("go back", "I go back.", "You go back."),
        "only" => ("only one", "I have only one.", "You have only one."),
        "very" => ("very big", "It is very big.", "This is very big."),
        "same" => ("the same", "It is the same.", "This is the same."),
        "last" => ("the last book", "This is the last book.", "The last book is here."),
        "kind" => ("a kind of book", "This is a kind of book.", "It is a kind of book."),
        "likely" => ("likely", "It is likely.", "That is likely."),
        "long term" => ("long term", "It is long term.", "This is long term."),
        "tv" => ("watch TV", "I watch TV.", "You watch TV."),
        "internet" => ("use the internet", "I use the internet.", "You use the internet."),
        "dvd" => ("a DVD", "This is a DVD.", "I use a DVD."),
        "cd" => ("a CD", "This is a CD.", "I use a CD."),
        "data" => ("know about data", "I know about data.", "You know about data."),
        "planning" => ("know about planning", "I know about planning.", "You know about planning."),
        "funding" => ("know about funding", "I know about funding.", "You know about funding."),
        "certain" => ("am certain", "I am certain.", "You are certain."),
        "police" => ("see the police", "I see the police.", "You see the police."),
        "none" => ("see none", "I see none.", "You see none."),
        "neither" => ("neither one", "Neither one is here.", "I see neither one."),
        "maybe" => ("maybe the same", "Maybe it is the same.", "Maybe this is the same."),
        "highly" => ("highly important", "It is highly important.", "This is highly important."),
        "fully" => ("fully ready", "It is fully ready.", "This is fully ready."),
        "initially" => ("initially go out", "Initially, I go out.", "Initially, you come in."),
        "thus" => ("thus", "Thus, I go.", "Thus, you come."),
        "cruel" => ("a cruel person", "He is a cruel person.", "She is a cruel person."),
        "involved" => ("involved in this", "I am involved in this.", "You are involved in it."),
        "opposed" => ("opposed to this", "I am opposed to this.", "You are opposed to it."),
        "depressed" => ("am depressed", "I am depressed.", "He is depressed."),
        "agenda" => ("the agenda", "This is the agenda.", "I see the agenda."),
        "aspect" => ("an aspect of this", "This is an aspect of this.", "I see an aspect of it."),
        "extent" => ("the extent", "This is the extent.", "I know the extent."),

        // Plural and collective nouns.
        "clothes" => ("these clothes", "These are clothes.", "I see clothes."),
        "jeans" => ("these jeans", "These are jeans.", "I see jeans."),
        "pants" => ("these pants", "These are pants.", "I see pants."),
        "goods" => ("these goods", "These are goods.", "I see goods."),
        "arms" => ("these arms", "These are arms.", "I see arms."),
        "people" => ("these people", "These are people.", "I see people."),
        "trousers" => ("these trousers", "These are trousers.", "I see trousers."),
        "means" => ("a means", "This is a means.", "It is a means."),
        "species" => ("a species", "This is a species.", "It is a species."),

        // Determiners and quantifiers.
        "another" => ("another book", "This is another book.", "I see another book."),
        "each" => ("each person", "Each person is here.", "I see each person."),
        "few" => ("a few books", "I see a few books.", "You have a few books."),
        "both" => ("both books", "Both books are here.", "I see both books."),
        "either" => ("either book", "Either book is here.", "I can have either book."),
        "several" => ("several books", "I see several books.", "You have several books."),
        "enough" => ("enough food", "I have enough food.", "You have enough food."),
        "many" => ("many books", "I have many books.", "You see many books."),
        "much" => ("much food", "I do not have much food.", "You do not have much food."),
        "little" => ("a little food", "I have a little food.", "You have a little food."),
        "various" => ("various books", "I see various books.", "You have various books."),
        "whole" => ("the whole book", "I see the whole book.", "You have the whole book."),
        "according" => ("according to", "This is according to the book.", "It is according to the word."),

        // Adjectives that are normally used predicatively.
        "awake" => ("am awake", "I am awake.", "You are awake."),
        "sure" => ("am sure", "I am sure.", "You are sure."),
        "sorry" => ("am sorry", "I am sorry.", "You are sorry."),
        "born" => ("is born", "A boy is born.", "A girl is born."),
        "afraid" => ("am afraid", "I am afraid.", "You are afraid."),
        "alone" => ("am alone", "I am alone.", "You are alone."),
        "alive" => ("is alive", "He is alive.", "She is alive."),
        "asleep" => ("is asleep", "He is asleep.", "She is asleep."),
        "aware" => ("am aware", "I am aware.", "You are aware."),
        "unable" => ("am unable", "I am unable.", "You are unable."),
        "glad" => ("am glad", "I am glad.", "You are glad."),
        "ashamed" => ("am ashamed", "I am ashamed.", "You are ashamed."),
        "worth" => ("worth it", "It is worth it.", "This is worth it."),
        "ill" => ("am ill", "I am ill.", "You are ill."),
        "well" => ("am well", "I am well.", "You are well."),

        // Verbs that naturally take a person as object.
        "persuade" => ("persuade him", "I can persuade him.", "You can persuade her."),
        "convince" => ("convince him", "I can convince him.", "You can convince her."),
        "remind" => ("remind him", "I can remind him.", "You can remind her."),
        "warn" => ("warn him", "I can warn him.", "You can warn her."),
        "inform" => ("inform him", "I can inform him.", "You can inform her."),
        "encourage" => ("encourage him", "I can encourage him.", "You can encourage her."),
        "invite" => ("invite him", "I can invite him.", "You can invite her."),
        "greet" => ("greet him", "I can greet him.", "You can greet her."),
        "educate" => ("educate him", "I can educate him.", "You can educate her."),
        "annoy" => ("annoy him", "I can annoy him.", "You can annoy her."),
        "frighten" => ("frighten him", "I can frighten him.", "You can frighten her."),
        "confuse" => ("confuse him", "I can confuse him.", "You can confuse her."),
        "inspire" => ("inspire him", "I can inspire him.", "You can inspire her."),
        "accuse" => ("accuse him", "I can accuse him.", "You can accuse her."),
        "accompany" => ("accompany him", "I can accompany him.", "You can accompany her."),
        "offend" => ("offend him", "I offend him.", "You offend her."),
        "bother" => ("bother him", "I can bother him.", "You can bother her."),
        "owe" => ("owe him", "I owe him.", "You owe her."),
        "thank" => ("thank him", "I can thank him.", "You can thank her."),
        "ask" => ("ask him", "I can ask him.", "You can ask her."),
        "help" => ("help him", "I can help him.", "You can help her."),
        "teach" => ("teach him", "I can teach him.", "You can teach her."),
        "show" => ("show him this", "I can show him this.", "You can show her this."),
        "tell" => ("tell him this", "I can tell him this.", "You can tell her this."),
        "give" => ("give him this", "I can give him this.", "You can give her this."),
        "send" => ("send him this", "I can send him this.", "You can send her this."),
        "offer" => ("offer him this", "I can offer him this.", "You can offer her this."),
        "pay" => ("pay him", "I can pay him.", "You can pay her."),
        "promise" => ("promise him this", "I can promise him this.", "You can promise her this."),
        _ => {
            return dynamic_person_adjective_template(&lower)
                .or_else(|| dynamic_common_count_noun_template(word, &lower))
                .or_else(|| dynamic_adverb_template(&lower));
        }
    };

    Some(tuple(fixed.0, fixed.1, fixed.2))
}

fn dynamic_person_adjective_template(word: &str) -> Option<(String, String, String)> {
    if PERSON_STATE_ADJECTIVES.contains(&word) {
        return Some(tuple(
            &format!("am {word}"),
            &format!("I am {word}."),
            &format!("You are {word}."),
        ));
    }
    None
}

fn dynamic_common_count_noun_template(display: &str, lower: &str) -> Option<(String, String, String)> {
    if COMMON_COUNT_NOUNS.contains(&lower) {
        let article = indefinite_article(display);
        return Some(tuple(
            &format!("{article} {display}"),
            &format!("This is {article} {display}."),
            &format!("I have {article} {display}."),
        ));
    }
    None
}

fn dynamic_adverb_template(word: &str) -> Option<(String, String, String)> {
    if MANNER_ADVERBS.contains(&word) {
        return Some(tuple(
            &format!("do it {word}"),
            &format!("I do it {word}."),
            &format!("You do it {word}."),
        ));
    }
    if FREQUENCY_ADVERBS.contains(&word) {
        return Some(tuple(
            &format!("{word} go out"),
            &format!("I {word} go out."),
            &format!("You {word} come in."),
        ));
    }
    if DIRECTION_ADVERBS.contains(&word) {
        return Some(tuple(
            &format!("go {word}"),
            &format!("I go {word}."),
            &format!("You come {word}."),
        ));
    }
    if SAME_DEGREE_ADVERBS.contains(&word) || PROBABILITY_ADVERBS.contains(&word) {
        return Some(tuple(
            &format!("{word} the same"),
            &format!("It is {word} the same."),
            &format!("This is {word} the same."),
        ));
    }
    if BIG_DEGREE_ADVERBS.contains(&word) {
        return Some(tuple(
            &format!("{word} important"),
            &format!("It is {word} important."),
            &format!("This is {word} important."),
        ));
    }
    if SEQUENCE_ADVERBS.contains(&word) {
        let mut capitalized = word.to_owned();
        if let Some(first) = capitalized.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        return Some(tuple(
            &format!("{word} go out"),
            &format!("{capitalized}, I go out."),
            &format!("{capitalized}, you come in."),
        ));
    }
    match word {
        "furthermore" => Some(tuple("furthermore", "I go out. Furthermore, you come in.", "You come in. Furthermore, I go out.")),
        "however" => Some(tuple("however", "I go out. However, you do not.", "You come in. However, I do not.")),
        "indeed" => Some(tuple("indeed", "It is indeed the same.", "This is indeed the same.")),
        "instead" => Some(tuple("instead", "I do not go. You go instead.", "You do not come. I come instead.")),
        "nevertheless" => Some(tuple("nevertheless", "I go out. Nevertheless, you do not.", "You come in. Nevertheless, I do not.")),
        "otherwise" => Some(tuple("otherwise", "I go out. Otherwise, you go out.", "You come in. Otherwise, I come in.")),
        "therefore" => Some(tuple("therefore", "You are here. Therefore, I go.", "I am here. Therefore, you come.")),
        _ => None,
    }
}

fn indefinite_article(word: &str) -> &'static str {
    let normalized = word.to_ascii_lowercase();
    if matches!(normalized.as_str(), "hour" | "honest") {
        return "an";
    }
    if matches!(normalized.as_str(), "university" | "user" | "use" | "euro") {
        return "a";
    }
    match normalized.chars().find(|character| character.is_ascii_alphabetic()) {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn tuple(phrase: &str, first: &str, second: &str) -> (String, String, String) {
    (phrase.to_owned(), first.to_owned(), second.to_owned())
}
