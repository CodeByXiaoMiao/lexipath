use crate::course::{CoursePack, Question, Reading, SentenceItem, WordItem};

pub fn polish_generated_content(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id == "foundation-words" {
            continue;
        }

        for lesson in &mut stage.lessons {
            let mut reading_sentences = Vec::with_capacity(lesson.new_words.len() * 2);
            let mut sentence_items = Vec::with_capacity(lesson.new_words.len());
            let mut questions = Vec::with_capacity(lesson.new_words.len());

            for word in &mut lesson.new_words {
                word.text = clean_display_word(&word.text);
                apply_lexicon_override(word);
                let template = template_for(&word.text, &word.meaning);
                word.phrase = template.phrase;
                word.example = template.first.clone();
                sentence_items.push(SentenceItem {
                    text: template.first.clone(),
                    meaning: word.meaning.clone(),
                });
                reading_sentences.push(template.first);
                reading_sentences.push(template.second);
            }

            let answer_options = sentence_items
                .iter()
                .map(|sentence| sentence.text.clone())
                .collect::<Vec<_>>();
            for (index, word) in lesson.new_words.iter().enumerate() {
                questions.push(Question {
                    prompt: format!("选择正确使用“{}”对应词义的句子。", word.meaning),
                    options: answer_options.clone(),
                    correct_index: index,
                });
            }

            lesson.sentences = sentence_items;
            lesson.reading = Reading {
                title: "配套受控阅读".to_owned(),
                sentences: reading_sentences,
                questions,
            };
        }
    }
}

struct Template {
    phrase: String,
    first: String,
    second: String,
}

fn template_for(word: &str, meaning: &str) -> Template {
    if let Some(template) = fixed_template(word) {
        return template;
    }

    let normalized = word.to_ascii_lowercase();
    if is_calendar_word(&normalized) {
        return make(
            word,
            &format!("It is {word}."),
            &format!("It is {word} now."),
        );
    }

    match lexical_class(&normalized, meaning) {
        LexicalClass::CountNoun => {
            let article = indefinite_article(word);
            make(
                &format!("{article} {word}"),
                &format!("This is {article} {word}."),
                &format!("I have {article} {word}."),
            )
        }
        LexicalClass::NoArticleNoun => {
            if is_plural_noun(&normalized) {
                make(
                    word,
                    &format!("These are {word}."),
                    &format!("I see {word}."),
                )
            } else {
                make(
                    word,
                    &format!("This is {word}."),
                    &format!("I know about {word}."),
                )
            }
        }
        LexicalClass::TransitiveVerb => make(
            &format!("{word} it"),
            &format!("I can {word} it."),
            &format!("You can {word} it."),
        ),
        LexicalClass::IntransitiveVerb => make(
            &format!("can {word}"),
            &format!("I can {word}."),
            &format!("You can {word}."),
        ),
        LexicalClass::Adjective => make(
            &format!("is {word}"),
            &format!("It is {word}."),
            &format!("This is {word}."),
        ),
        LexicalClass::Adverb => make(
            &format!("do it {word}"),
            &format!("I can do it {word}."),
            &format!("You can do it {word}."),
        ),
        LexicalClass::Number => make(
            word,
            &format!("I have {word}."),
            &format!("You have {word}."),
        ),
        LexicalClass::Other => make(
            &format!("the word {word}"),
            &format!("The word is {word}."),
            &format!("I say {word}."),
        ),
    }
}

fn fixed_template(word: &str) -> Option<Template> {
    let word = word.to_ascii_lowercase();
    let values = match word.as_str() {
        "be" => ("can be", "I can be here.", "You can be there."),
        "do" => ("do this", "I can do this.", "You can do it."),
        "have" => ("have a book", "I have a book.", "You have a book."),
        "say" => ("say this", "I can say this.", "You can say it."),
        "may" => ("may go", "I may go.", "You may come."),
        "might" => ("might go", "I might go.", "You might come."),
        "will" => ("will go", "I will go.", "You will come."),
        "would" => ("would go", "I would go.", "You would come."),
        "could" => ("could go", "I could go.", "You could come."),
        "should" => ("should go", "I should go.", "You should come."),
        "must" => ("must go", "I must go.", "You must come."),
        "cannot" => ("cannot go", "I cannot go.", "You cannot come."),
        "about" => ("about this", "This is about a book.", "This is about food."),
        "across" => ("across the room", "I go across the room.", "You come across the room."),
        "after" => ("after this", "I go after this.", "You come after this."),
        "against" => ("against this", "I am against this.", "You are against it."),
        "among" => ("among people", "I am among people.", "You are among people."),
        "before" => ("before this", "I go before this.", "You come before this."),
        "between" => ("between you and me", "It is between you and me.", "This is between him and her."),
        "by" => ("by me", "This book is by me.", "This book is by you."),
        "from" => ("from here", "I come from here.", "You come from there."),
        "off" => ("go off", "I can go off.", "You can go off."),
        "over" => ("go over", "I can go over this.", "You can go over it."),
        "through" => ("through the room", "I go through the room.", "You come through the room."),
        "to" => ("go to", "I go to you.", "You come to me."),
        "with" => ("with you", "I am with you.", "You are with me."),
        "without" => ("without you", "I can go without you.", "You can go without me."),
        "as" => ("as I do", "You do as I do.", "I do as you do."),
        "for" => ("for you", "This is for you.", "It is for me."),
        "of" => ("one of two", "This is one of two.", "It is one of three."),
        "till" => ("till night", "I am here till night.", "You are there till morning."),
        "all" => ("all people", "All people are here.", "We are all here."),
        "any" => ("any food", "Do you have any food?", "I do not have any food."),
        "every" => ("every day", "I go every day.", "You come every day."),
        "other" => ("the other book", "This is the other book.", "The other book is there."),
        "some" => ("some food", "I have some food.", "You have some food."),
        "such" => ("such a book", "This is such a book.", "It is such a big book."),
        "that" => ("that book", "That is a book.", "That book is there."),
        "who" => ("who is here", "Who is here?", "Who is there?"),
        "because" => ("because you come", "I go because you come.", "You go because I come."),
        "but" => ("but not", "I go, but you do not.", "You come, but I do not."),
        "or" => ("you or me", "You or me?", "He or she?"),
        "if" => ("if you come", "I go if you come.", "You go if I come."),
        "though" => ("though it is small", "I like it, though it is small.", "You like it, though it is big."),
        "while" => ("while you are here", "I go while you are here.", "You go while I am here."),
        "how" => ("how do", "How do I do it?", "How do you do it?"),
        "when" => ("when you come", "When do you come?", "I go when you come."),
        "where" => ("where it is", "Where is it?", "Where are you?"),
        "again" => ("go again", "I can go again.", "You can come again."),
        "ever" => ("ever here", "Are you ever here?", "Am I ever there?"),
        "far" => ("far", "It is far.", "This is far."),
        "forward" => ("go forward", "I can go forward.", "You can go forward."),
        "out" => ("go out", "I can go out.", "You can go out."),
        "still" => ("still here", "I am still here.", "You are still there."),
        "then" => ("then go", "I go, then you go.", "You come, then I come."),
        "within" => ("within this room", "It is within this room.", "I am within this room."),
        "despite" => ("despite this", "I go despite this.", "You come despite it."),
        "towards" => ("towards me", "You come towards me.", "I go towards you."),
        "thanks" => ("say thanks", "I say thanks.", "You say thanks."),
        "nothing" => ("see nothing", "I see nothing.", "You see nothing."),
        "tonight" => ("go tonight", "I go tonight.", "You come tonight."),
        "midnight" => ("at midnight", "It is midnight.", "I am here at midnight."),
        "mum" => ("my mum", "She is my mum.", "I see my mum."),
        "daughter" => ("a daughter", "She is a daughter.", "I have a daughter."),
        "uncle" => ("my uncle", "He is my uncle.", "I see my uncle."),
        "female" => ("female", "She is female.", "It is female."),
        "library" => ("go to a library", "I go to a library.", "You go to a library."),
        "pain" => ("feel pain", "I feel pain.", "You feel pain."),
        "shake" => ("shake it", "I can shake it.", "You can shake it."),
        "middle" => ("the middle", "This is the middle.", "I see the middle."),
        "heat" => ("heat", "This is heat.", "I feel heat."),
        "sun" => ("the sun", "This is the sun.", "I see the sun."),
        "lip" => ("my lip", "This is my lip.", "I touch my lip."),
        "birth" => ("birth", "This is birth.", "I know about birth."),
        "breath" => ("a breath", "I take a breath.", "You take a breath."),
        "noise" => ("noise", "This is noise.", "I hear noise."),
        "fold" => ("fold it", "I can fold it.", "You can fold it."),
        "period" => ("a period of time", "This is a period of time.", "I see a period of time."),
        "type" => ("a type of book", "This is a type of book.", "I see a type of book."),
        "routine" => ("a routine", "This is my routine.", "You have a routine."),
        _ => return None,
    };
    Some(make(values.0, values.1, values.2))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalClass {
    CountNoun,
    NoArticleNoun,
    TransitiveVerb,
    IntransitiveVerb,
    Adjective,
    Adverb,
    Number,
    Other,
}

fn lexical_class(word: &str, meaning: &str) -> LexicalClass {
    let normalized = meaning.trim().to_ascii_lowercase();
    if normalized.starts_with("adv.") {
        return LexicalClass::Adverb;
    }
    if normalized.starts_with("adj.") || normalized.starts_with("a.") {
        return LexicalClass::Adjective;
    }
    if normalized.starts_with("vt.") {
        return LexicalClass::TransitiveVerb;
    }
    if normalized.starts_with("vi.") || normalized.starts_with("v.") {
        return LexicalClass::IntransitiveVerb;
    }
    if normalized.starts_with("num.") {
        return LexicalClass::Number;
    }
    if normalized.starts_with("n.") {
        return if is_no_article_noun(word) {
            LexicalClass::NoArticleNoun
        } else {
            LexicalClass::CountNoun
        };
    }
    LexicalClass::Other
}

fn apply_lexicon_override(word: &mut WordItem) {
    let normalized = word.text.to_ascii_lowercase();
    let Some((ipa, meaning)) = (match normalized.as_str() {
        "ant" => Some(("/ænt/", "n. 蚂蚁")),
        "able" => Some(("/ˈeɪbl/", "adj. 能够的；有能力的")),
        "form" => Some(("/fɔːrm/", "n. 形式；表格")),
        "head" => Some(("/hed/", "n. 头；负责人")),
        "arch" => Some(("/ɑːrtʃ/", "n. 拱门；拱形")),
        "meal" => Some(("/miːl/", "n. 一餐；饭")),
        "phone" => Some(("/foʊn/", "n. 电话")),
        "speak" => Some(("/spiːk/", "vi. 说话；讲话")),
        "them" => Some(("/ðem/", "pron. 他们；她们；它们（宾格）")),
        "style" => Some(("/staɪl/", "n. 风格；样式")),
        "age" => Some(("/eɪdʒ/", "n. 年龄；时代")),
        "later" => Some(("/ˈleɪtər/", "adv. 后来；较晚")),
        "its" => Some(("/ɪts/", "pron. 它的")),
        "ability" => Some(("/əˈbɪləti/", "n. 能力")),
        "specific" => Some(("/spəˈsɪfɪk/", "adj. 特定的；明确的")),
        "penny" => Some(("/ˈpeni/", "n. 便士")),
        "gate" => Some(("/ɡeɪt/", "n. 大门；出入口")),
        "app" => Some(("/æp/", "n. 应用程序")),
        "ad" => Some(("/æd/", "n. 广告")),
        "grade" => Some(("/ɡreɪd/", "n. 等级；成绩")),
        "path" => Some(("/pæθ/", "n. 小路；路径")),
        "logical" => Some(("/ˈlɑːdʒɪkl/", "adj. 合乎逻辑的")),
        "resist" => Some(("/rɪˈzɪst/", "vt. 抵抗；忍住")),
        "us" => Some(("/ʌs/", "pron. 我们；咱们")),
        "read" => Some(("/riːd/", "v. 阅读；读")),
        "neat" => Some(("/niːt/", "adj. 整洁的；利落的")),
        "statue" => Some(("/ˈstætʃuː/", "n. 雕像")),
        "employ" => Some(("/ɪmˈplɔɪ/", "vt. 雇用；使用")),
        "elect" => Some(("/ɪˈlekt/", "vt. 选举；推选")),
        _ => None,
    }) else {
        return;
    };
    word.ipa = ipa.to_owned();
    word.meaning = meaning.to_owned();
}

fn make(phrase: &str, first: &str, second: &str) -> Template {
    Template {
        phrase: phrase.to_owned(),
        first: first.to_owned(),
        second: second.to_owned(),
    }
}

fn clean_display_word(value: &str) -> String {
    value
        .trim()
        .trim_matches(|character: char| {
            !character.is_ascii_alphanumeric() && character != '\'' && character != '-'
        })
        .trim_start_matches('-')
        .trim_end_matches('-')
        .trim_end_matches('.')
        .to_owned()
}

fn indefinite_article(word: &str) -> &'static str {
    let normalized = word.to_ascii_lowercase();
    if matches!(normalized.as_str(), "hour" | "honest" | "honour") {
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

fn is_calendar_word(word: &str) -> bool {
    matches!(
        word,
        "monday"
            | "tuesday"
            | "wednesday"
            | "thursday"
            | "friday"
            | "saturday"
            | "sunday"
            | "january"
            | "february"
            | "march"
            | "april"
            | "may"
            | "june"
            | "july"
            | "august"
            | "september"
            | "october"
            | "november"
            | "december"
    )
}

fn is_no_article_noun(word: &str) -> bool {
    is_plural_noun(word)
        || matches!(
            word,
            "access"
                | "advice"
                | "air"
                | "alcohol"
                | "anger"
                | "art"
                | "beef"
                | "beer"
                | "biology"
                | "blood"
                | "brass"
                | "bread"
                | "business"
                | "butter"
                | "cancer"
                | "cash"
                | "chalk"
                | "chemistry"
                | "climate"
                | "coffee"
                | "conduct"
                | "cooking"
                | "cotton"
                | "damage"
                | "dancing"
                | "data"
                | "death"
                | "dirt"
                | "economy"
                | "education"
                | "energy"
                | "engineering"
                | "enthusiasm"
                | "equipment"
                | "evil"
                | "feedback"
                | "folk"
                | "food"
                | "fun"
                | "fur"
                | "furniture"
                | "gas"
                | "geography"
                | "gold"
                | "grass"
                | "ground"
                | "health"
                | "hockey"
                | "homework"
                | "honour"
                | "housing"
                | "ice"
                | "ink"
                | "information"
                | "jazz"
                | "jewellery"
                | "juice"
                | "knowledge"
                | "labour"
                | "laughter"
                | "leather"
                | "linen"
                | "literature"
                | "luggage"
                | "mail"
                | "marketing"
                | "mathematics"
                | "meat"
                | "medicine"
                | "media"
                | "milk"
                | "money"
                | "mud"
                | "music"
                | "news"
                | "oil"
                | "paper"
                | "petrol"
                | "plastic"
                | "poetry"
                | "police"
                | "privacy"
                | "produce"
                | "research"
                | "rice"
                | "rubbish"
                | "rugby"
                | "salt"
                | "sand"
                | "silk"
                | "smoke"
                | "soap"
                | "soccer"
                | "space"
                | "steel"
                | "steam"
                | "stress"
                | "stuff"
                | "sugar"
                | "sympathy"
                | "tea"
                | "tennis"
                | "thunder"
                | "traffic"
                | "underwear"
                | "violence"
                | "waste"
                | "water"
                | "wax"
                | "weather"
                | "wildlife"
                | "wood"
                | "wool"
                | "work"
        )
}

fn is_plural_noun(word: &str) -> bool {
    if matches!(
        word,
        "analysis"
            | "apparatus"
            | "basis"
            | "boss"
            | "brass"
            | "business"
            | "canvas"
            | "campus"
            | "class"
            | "crisis"
            | "dress"
            | "emphasis"
            | "gas"
            | "glass"
            | "grass"
            | "guess"
            | "kiss"
            | "loss"
            | "mass"
            | "mess"
            | "news"
            | "press"
            | "process"
            | "status"
            | "stress"
            | "success"
            | "tennis"
            | "virus"
    ) {
        return false;
    }
    if word.ends_with("ss") || word.ends_with("us") || word.ends_with("is") || word.ends_with("ics") {
        return false;
    }
    word.ends_with('s')
        && !matches!(
            word,
            "means"
                | "series"
                | "species"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_dictionary_affix_artifacts() {
        assert_eq!(clean_display_word("-phone"), "phone");
        assert_eq!(clean_display_word("app."), "app");
    }

    #[test]
    fn repairs_affix_dictionary_meanings() {
        let mut word = WordItem {
            id: "phone".to_owned(),
            text: "phone".to_owned(),
            ipa: "bad".to_owned(),
            meaning: "suffix".to_owned(),
            phrase: String::new(),
            example: String::new(),
        };
        apply_lexicon_override(&mut word);
        assert_eq!(word.ipa, "/foʊn/");
        assert!(word.meaning.starts_with("n."));
    }

    #[test]
    fn repairs_known_bad_dictionary_senses() {
        let mut word = WordItem {
            id: "us".to_owned(),
            text: "us".to_owned(),
            ipa: "bad".to_owned(),
            meaning: "pron. 我们\n[经] 美国".to_owned(),
            phrase: String::new(),
            example: String::new(),
        };
        apply_lexicon_override(&mut word);
        assert_eq!(word.meaning, "pron. 我们；咱们");
    }

    #[test]
    fn calendar_words_use_natural_frames() {
        let template = template_for("Friday", "n. 星期五");
        assert_eq!(template.first, "It is Friday.");
    }

    #[test]
    fn transitive_verbs_receive_an_object() {
        let template = template_for("accept", "vt. 接受");
        assert_eq!(template.first, "I can accept it.");
    }

    #[test]
    fn mass_nouns_do_not_receive_an_indefinite_article() {
        let template = template_for("information", "n. 信息");
        assert_eq!(template.first, "This is information.");
        assert_eq!(template.second, "I know about information.");
    }

    #[test]
    fn singular_words_ending_in_s_are_not_treated_as_plurals() {
        let template = template_for("process", "n. 过程");
        assert_eq!(template.first, "This is a process.");
    }

    #[test]
    fn expanded_mass_nouns_are_not_given_articles() {
        let template = template_for("blood", "n. 血液");
        assert_eq!(template.first, "This is blood.");
    }
}
