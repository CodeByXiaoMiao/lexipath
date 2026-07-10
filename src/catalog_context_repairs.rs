use crate::course::Lesson;

pub fn apply_context_repairs(lesson: &mut Lesson) {
    for index in 0..lesson.new_words.len() {
        let lower = lesson.new_words[index].text.to_ascii_lowercase();
        let Some(repair) = context_repair(&lower) else {
            continue;
        };

        {
            let word = &mut lesson.new_words[index];
            if let Some(meaning) = repair.meaning {
                word.meaning = meaning.to_owned();
            }
            word.phrase = repair.phrase.to_owned();
            word.example = repair.first.to_owned();
        }
        if let Some(sentence) = lesson.sentences.get_mut(index) {
            sentence.text = repair.first.to_owned();
        }
        let reading_index = index * 2;
        if let Some(sentence) = lesson.reading.sentences.get_mut(reading_index) {
            *sentence = repair.first.to_owned();
        }
        if let Some(sentence) = lesson.reading.sentences.get_mut(reading_index + 1) {
            *sentence = repair.second.to_owned();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ContextRepair {
    meaning: Option<&'static str>,
    phrase: &'static str,
    first: &'static str,
    second: &'static str,
}

fn context_repair(word: &str) -> Option<ContextRepair> {
    let repair = match word {
        "since" => ContextRepair {
            meaning: None,
            phrase: "since you are here",
            first: "Since you are here, I can go.",
            second: "Since I am here, you can go.",
        },
        "minute" => ContextRepair {
            meaning: None,
            phrase: "one minute",
            first: "I have one minute.",
            second: "You have one minute.",
        },
        "mine" => ContextRepair {
            meaning: Some("pron. 我的"),
            phrase: "is mine",
            first: "This book is mine.",
            second: "The book is mine.",
        },
        "trip" => ContextRepair {
            meaning: None,
            phrase: "go on a trip",
            first: "I go on a trip.",
            second: "You go on a trip.",
        },
        "rise" => ContextRepair {
            meaning: Some("v. 上升；升起"),
            phrase: "can rise",
            first: "It can rise.",
            second: "This can rise.",
        },
        "soft" => ContextRepair {
            meaning: None,
            phrase: "is soft",
            first: "It is soft.",
            second: "This is soft.",
        },
        "involve" => ContextRepair {
            meaning: None,
            phrase: "involve work",
            first: "This can involve work.",
            second: "It can involve work.",
        },
        "surround" => ContextRepair {
            meaning: None,
            phrase: "surround this",
            first: "They can surround this.",
            second: "We can surround it.",
        },
        _ => return None,
    };
    Some(repair)
}
