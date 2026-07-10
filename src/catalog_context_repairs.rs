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
    REPAIRS
        .iter()
        .find(|record| record.0 == word)
        .map(|record| ContextRepair {
            meaning: record.1,
            phrase: record.2,
            first: record.3,
            second: record.4,
        })
}

type RepairRecord = (
    &'static str,
    Option<&'static str>,
    &'static str,
    &'static str,
    &'static str,
);

const REPAIRS: &[RepairRecord] = &[
    ("since", None, "since you are here", "Since you are here, I can go.", "Since I am here, you can go."),
    ("minute", Some("n. \u{5206}\u{949f}"), "one minute", "I have one minute.", "You have one minute."),
    ("mine", Some("pron. \u{6211}\u{7684}"), "is mine", "This book is mine.", "The book is mine."),
    ("feeling", Some("n. \u{611f}\u{89c9}"), "a feeling", "I have a feeling.", "You have a feeling."),
    ("driving", Some("n. \u{9a7e}\u{9a76}"), "driving", "I like driving.", "You like driving."),
    ("copper", Some("n. \u{94dc}"), "copper", "This is copper.", "I have copper."),
    ("iron", Some("n. \u{94c1}"), "iron", "This is iron.", "I have iron."),
    ("cent", Some("n. \u{7f8e}\u{5206}"), "a cent", "This is a cent.", "I have a cent."),
    ("bar", Some("n. \u{9152}\u{5427}"), "a bar", "This is a bar.", "I see a bar."),
    ("trip", None, "go on a trip", "I go on a trip.", "You go on a trip."),
    ("human", Some("n. \u{4eba}"), "a human", "He is a human.", "She is a human."),
    ("individual", Some("n. \u{4e2a}\u{4eba}"), "an individual", "He is an individual.", "She is an individual."),
    ("meet", Some("v. \u{9047}\u{89c1}\u{ff1b}\u{4f1a}\u{9762}"), "meet me", "You can meet me.", "I can meet you."),
    ("borrow", Some("v. \u{501f}\u{5165}"), "borrow a book", "I can borrow a book.", "You can borrow a book."),
    ("lend", Some("v. \u{501f}\u{51fa}"), "lend a book", "I can lend a book.", "You can lend a book."),
    ("yeah", Some("interj. \u{662f}\u{7684}"), "say \"yeah\"", "I say \"yeah\".", "You say \"yeah\"."),
    ("upstairs", Some("adv. \u{5728}\u{697c}\u{4e0a}"), "upstairs", "I am upstairs.", "You are upstairs."),
    ("downstairs", Some("adv. \u{5728}\u{697c}\u{4e0b}"), "downstairs", "I am downstairs.", "You are downstairs."),
    ("apply", Some("v. \u{7533}\u{8bf7}"), "can apply", "I can apply.", "You can apply."),
    ("solution", Some("n. \u{89e3}\u{51b3}\u{529e}\u{6cd5}"), "a solution", "This is a solution.", "I have a solution."),
    ("none", Some("pron. \u{4e00}\u{4e2a}\u{4e5f}\u{6ca1}\u{6709}"), "none", "I have none.", "You have none."),
    ("recording", Some("n. \u{5f55}\u{97f3}"), "a recording", "This is a recording.", "I have a recording."),
    ("flying", Some("n. \u{98de}\u{884c}"), "flying", "I like flying.", "You like flying."),
    ("latest", Some("adj. \u{6700}\u{65b0}\u{7684}"), "the latest one", "This is the latest one.", "It is the latest one."),
    ("global", Some("adj. \u{5168}\u{7403}\u{7684}"), "is global", "It is global.", "This is global."),
    ("tiny", Some("adj. \u{6781}\u{5c0f}\u{7684}"), "is tiny", "It is tiny.", "This is tiny."),
    ("related", Some("adj. \u{76f8}\u{5173}\u{7684}"), "related to this", "It is related to this.", "This is related to it."),
    ("sensible", Some("adj. \u{660e}\u{667a}\u{7684}"), "is sensible", "It is sensible.", "This is sensible."),
    ("advanced", Some("adj. \u{9ad8}\u{7ea7}\u{7684}\u{ff1b}\u{5148}\u{8fdb}\u{7684}"), "is advanced", "It is advanced.", "This is advanced."),
    ("fighting", Some("n. \u{6253}\u{6597}"), "fighting", "I see fighting.", "You see fighting."),
    ("indoor", Some("adj. \u{5ba4}\u{5185}\u{7684}"), "an indoor game", "This is an indoor game.", "It is an indoor game."),
    ("spoken", Some("adj. \u{53e3}\u{8bed}\u{7684}"), "a spoken word", "This is a spoken word.", "It is a spoken word."),
    ("spicy", Some("adj. \u{8f9b}\u{8fa3}\u{7684}"), "is spicy", "It is spicy.", "This is spicy."),
    ("located", Some("adj. \u{4f4d}\u{4e8e}"), "located here", "It is located here.", "This is located here."),
    ("long term", Some("adj. \u{957f}\u{671f}\u{7684}"), "is long term", "This plan is long term.", "The plan is long term."),
    ("creature", Some("n. \u{751f}\u{7269}"), "a creature", "This is a creature.", "I see a creature."),
    ("critical", Some("adj. \u{5173}\u{952e}\u{7684}"), "is critical", "It is critical.", "This is critical."),
    ("vital", Some("adj. \u{81f3}\u{5173}\u{91cd}\u{8981}\u{7684}"), "is vital", "It is vital.", "This is vital."),
    ("secure", Some("adj. \u{5b89}\u{5168}\u{7684}"), "is secure", "It is secure.", "This is secure."),
    ("severe", Some("adj. \u{4e25}\u{91cd}\u{7684}"), "is severe", "It is severe.", "This is severe."),
    ("intense", Some("adj. \u{5f3a}\u{70c8}\u{7684}"), "is intense", "It is intense.", "This is intense."),
    ("virtual", Some("adj. \u{865a}\u{62df}\u{7684}"), "is virtual", "It is virtual.", "This is virtual."),
    ("minimum", Some("n. \u{6700}\u{4f4e}\u{9650}\u{5ea6}"), "the minimum", "This is the minimum.", "It is the minimum."),
    ("outer", Some("adj. \u{5916}\u{90e8}\u{7684}"), "the outer part", "This is the outer part.", "It is the outer part."),
    ("solar", Some("adj. \u{592a}\u{9633}\u{80fd}\u{7684}"), "solar power", "This is solar power.", "It is solar power."),
    ("daily", Some("adj. \u{6bcf}\u{65e5}\u{7684}"), "daily work", "This is daily work.", "It is daily work."),
    ("financial", Some("adj. \u{8d22}\u{52a1}\u{7684}"), "financial work", "This is financial work.", "It is financial work."),
    ("previous", Some("adj. \u{5148}\u{524d}\u{7684}"), "the previous one", "This is the previous one.", "It is the previous one."),
    ("primary", Some("adj. \u{4e3b}\u{8981}\u{7684}"), "the primary reason", "This is the primary reason.", "It is the primary reason."),
    ("nuclear", Some("adj. \u{6838}\u{80fd}\u{7684}"), "nuclear power", "This is nuclear power.", "It is nuclear power."),
    ("technical", Some("adj. \u{6280}\u{672f}\u{7684}"), "technical work", "This is technical work.", "It is technical work."),
    ("academic", Some("adj. \u{5b66}\u{672f}\u{7684}"), "academic work", "This is academic work.", "It is academic work."),
    ("mental", Some("adj. \u{5fc3}\u{7406}\u{7684}"), "mental health", "This is mental health.", "I know about mental health."),
    ("educational", Some("adj. \u{6559}\u{80b2}\u{7684}"), "educational work", "This is educational work.", "It is educational work."),
    ("scientific", Some("adj. \u{79d1}\u{5b66}\u{7684}"), "scientific work", "This is scientific work.", "It is scientific work."),
    ("secondary", Some("adj. \u{4e2d}\u{5b66}\u{7684}\u{ff1b}\u{6b21}\u{8981}\u{7684}"), "a secondary school", "This is a secondary school.", "It is a secondary school."),
    ("written", Some("adj. \u{4e66}\u{9762}\u{7684}"), "written work", "This is written work.", "It is written work."),
    ("remote", Some("adj. \u{504f}\u{8fdc}\u{7684}"), "a remote area", "This is a remote area.", "It is a remote area."),
    ("aged", Some("adj. \u{5e74}\u{8001}\u{7684}"), "an aged person", "He is an aged person.", "She is an aged person."),
    ("repeated", Some("adj. \u{91cd}\u{590d}\u{7684}"), "repeated work", "This is repeated work.", "It is repeated work."),
    ("additional", Some("adj. \u{989d}\u{5916}\u{7684}"), "additional work", "This is additional work.", "It is additional work."),
    ("rural", Some("adj. \u{4e61}\u{6751}\u{7684}"), "a rural area", "This is a rural area.", "It is a rural area."),
    ("detailed", Some("adj. \u{8be6}\u{7ec6}\u{7684}"), "a detailed plan", "This is a detailed plan.", "It is a detailed plan."),
    ("external", Some("adj. \u{5916}\u{90e8}\u{7684}"), "the external part", "This is the external part.", "It is the external part."),
    ("visual", Some("adj. \u{89c6}\u{89c9}\u{7684}"), "visual art", "This is visual art.", "It is visual art."),
    ("artistic", Some("adj. \u{827a}\u{672f}\u{7684}"), "artistic work", "This is artistic work.", "It is artistic work."),
    ("associated", Some("adj. \u{76f8}\u{5173}\u{7684}"), "associated with this", "It is associated with this.", "This is associated with it."),
    ("intended", Some("adj. \u{9884}\u{5b9a}\u{7684}\u{ff1b}\u{6709}\u{610f}\u{7684}"), "intended for you", "It is intended for you.", "This is intended for me."),
    ("rise", Some("v. \u{4e0a}\u{5347}\u{ff1b}\u{5347}\u{8d77}"), "can rise", "It can rise.", "This can rise."),
    ("soft", None, "is soft", "It is soft.", "This is soft."),
    ("involve", None, "involve work", "This can involve work.", "It can involve work."),
    ("surround", None, "surround this", "They can surround this.", "We can surround it."),
];
