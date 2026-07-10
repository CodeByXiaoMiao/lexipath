use std::collections::HashMap;

use crate::catalog_meaning::learner_gloss;
use crate::course::{CoursePack, Lesson};

#[derive(Debug, Clone, Default)]
pub struct TranslationGuide {
    lexicon: HashMap<String, String>,
}

impl TranslationGuide {
    pub fn new(course: &CoursePack) -> Self {
        let mut lexicon = common_lexicon();
        for stage in &course.stages {
            for lesson in &stage.lessons {
                for word in &lesson.new_words {
                    lexicon
                        .entry(word.text.to_ascii_lowercase())
                        .or_insert_with(|| clean_gloss(learner_gloss(&word.meaning)));
                }
            }
        }
        Self { lexicon }
    }

    pub fn sentence(&self, lesson: &Lesson, english: &str) -> String {
        if let Some(manual) = curated_translation(lesson.id.as_str(), english) {
            return manual.to_owned();
        }
        if let Some(item) = lesson.sentences.iter().find(|item| item.text == english) {
            if looks_like_complete_translation(&item.meaning) {
                return item.meaning.clone();
            }
        }
        self.translate_controlled(english)
    }

    fn translate_controlled(&self, english: &str) -> String {
        let sentence = english.trim();
        let core = sentence
            .trim_end_matches(|character| matches!(character, '.' | '?' | '!'))
            .trim();
        let punctuation = if sentence.ends_with('?') {
            "？"
        } else if sentence.ends_with('!') {
            "！"
        } else {
            "。"
        };

        for (prefix, chinese) in [
            ("This is a ", "这是一个"),
            ("This is an ", "这是一个"),
            ("That is a ", "那是一个"),
            ("That is an ", "那是一个"),
            ("I have a ", "我有一个"),
            ("I have an ", "我有一个"),
            ("You have a ", "你有一个"),
            ("You have an ", "你有一个"),
            ("He is a ", "他是一个"),
            ("He is an ", "他是一个"),
            ("She is a ", "她是一个"),
            ("She is an ", "她是一个"),
        ] {
            if let Some(rest) = core.strip_prefix(prefix) {
                return format!("{chinese}{}{punctuation}", self.translate_phrase(rest));
            }
        }

        for (prefix, chinese) in [
            ("These are ", "这些是"),
            ("Those are ", "那些是"),
            ("This is ", "这是"),
            ("That is ", "那是"),
            ("It is ", "它是"),
            ("They are ", "他们是"),
            ("We are ", "我们是"),
            ("I am ", "我是"),
            ("You are ", "你是"),
            ("He is ", "他是"),
            ("She is ", "她是"),
        ] {
            if let Some(rest) = core.strip_prefix(prefix) {
                return format!("{chinese}{}{punctuation}", self.translate_phrase(rest));
            }
        }

        for (prefix, chinese) in [
            ("I know about ", "我了解"),
            ("You know about ", "你了解"),
            ("I see ", "我看见"),
            ("You see ", "你看见"),
            ("We see ", "我们看见"),
            ("They see ", "他们看见"),
            ("I have ", "我有"),
            ("You have ", "你有"),
            ("We have ", "我们有"),
            ("They have ", "他们有"),
        ] {
            if let Some(rest) = core.strip_prefix(prefix) {
                return format!("{chinese}{}{punctuation}", self.translate_phrase(rest));
            }
        }

        for (prefix, subject) in [
            ("I can ", "我能"),
            ("You can ", "你能"),
            ("We can ", "我们能"),
            ("They can ", "他们能"),
            ("He can ", "他能"),
            ("She can ", "她能"),
            ("It can ", "它能"),
            ("This can ", "这能"),
            ("I may ", "我可以"),
            ("You may ", "你可以"),
            ("I will ", "我将"),
            ("You will ", "你将"),
            ("I should ", "我应该"),
            ("You should ", "你应该"),
            ("I must ", "我必须"),
            ("You must ", "你必须"),
            ("I could ", "我可以"),
            ("You could ", "你可以"),
            ("I would ", "我会"),
            ("You would ", "你会"),
        ] {
            if let Some(rest) = core.strip_prefix(prefix) {
                return format!("{subject}{}{punctuation}", self.translate_action(rest));
            }
        }

        if let Some(rest) = core.strip_prefix("Do you have ") {
            return format!("你有{}吗？", self.translate_phrase(rest));
        }
        if let Some(rest) = core.strip_prefix("Do you like ") {
            return format!("你喜欢{}吗？", self.translate_phrase(rest));
        }
        if let Some(rest) = core.strip_prefix("Where is ") {
            return format!("{}在哪里？", self.translate_phrase(rest));
        }
        if core == "Where are you" {
            return "你在哪里？".to_owned();
        }
        if core == "Who is here" {
            return "谁在这里？".to_owned();
        }
        if core == "Who is there" {
            return "谁在那里？".to_owned();
        }
        if core == "Yes, I do" {
            return "是的，我喜欢。".to_owned();
        }
        if core == "No, I do not" {
            return "不，我不喜欢。".to_owned();
        }
        if core == "Why not" {
            return "为什么不呢？".to_owned();
        }

        format!("{}{punctuation}", self.translate_tokens(core))
    }

    fn translate_action(&self, value: &str) -> String {
        let mut words = value.split_whitespace();
        let Some(verb) = words.next() else {
            return String::new();
        };
        let verb_cn = self.lookup(verb);
        let rest = words.collect::<Vec<_>>().join(" ");
        if rest.is_empty() {
            return verb_cn;
        }
        let object = match rest.as_str() {
            "it" => "它".to_owned(),
            "this" => "这个".to_owned(),
            "that" => "那个".to_owned(),
            "me" => "我".to_owned(),
            "you" => "你".to_owned(),
            _ => self.translate_phrase(&rest),
        };
        format!("{verb_cn}{object}")
    }

    fn translate_phrase(&self, value: &str) -> String {
        let value = value.trim();
        if value.is_empty() {
            return String::new();
        }
        if let Some((left, right)) = value.split_once(" and ") {
            return format!("{}和{}", self.translate_phrase(left), self.translate_phrase(right));
        }
        if let Some((left, right)) = value.split_once(" or ") {
            return format!("{}或{}", self.translate_phrase(left), self.translate_phrase(right));
        }
        self.translate_tokens(value)
    }

    fn translate_tokens(&self, value: &str) -> String {
        value
            .split_whitespace()
            .map(|token| self.lookup(token))
            .collect::<Vec<_>>()
            .join("")
    }

    fn lookup(&self, token: &str) -> String {
        let normalized = token
            .trim_matches(|character: char| !character.is_ascii_alphabetic() && character != '\'')
            .to_ascii_lowercase();
        if normalized.is_empty() {
            return token.to_owned();
        }
        if let Some(value) = self.lexicon.get(&normalized) {
            return value.clone();
        }
        for candidate in morphology_candidates(&normalized) {
            if let Some(value) = self.lexicon.get(candidate.as_str()) {
                return value.clone();
            }
        }
        normalized
    }
}

fn clean_gloss(value: &str) -> String {
    value
        .split(|character| matches!(character, '；' | ';' | '，' | ','))
        .next()
        .unwrap_or(value)
        .trim()
        .trim_end_matches('。')
        .to_owned()
}

fn looks_like_complete_translation(value: &str) -> bool {
    let trimmed = value.trim();
    let has_chinese = trimmed
        .chars()
        .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character));
    let normalized_pos = [
        "n. ", "v. ", "vt. ", "vi. ", "adj. ", "adv. ", "prep. ", "conj. ",
        "pron. ", "det. ", "modal. ", "num. ", "interj. ", "word. ",
    ]
    .iter()
    .any(|prefix| trimmed.starts_with(prefix));
    has_chinese && !normalized_pos
}

fn morphology_candidates(word: &str) -> Vec<String> {
    let mut values = Vec::new();
    if let Some(stem) = word.strip_suffix("ies") {
        values.push(format!("{stem}y"));
    }
    if let Some(stem) = word.strip_suffix("ing") {
        values.push(stem.to_owned());
        if stem.len() > 2 {
            values.push(stem[..stem.len() - 1].to_owned());
        }
        values.push(format!("{stem}e"));
    }
    if let Some(stem) = word.strip_suffix("ed") {
        values.push(stem.to_owned());
        values.push(format!("{stem}e"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        values.push(stem.to_owned());
    }
    if let Some(stem) = word.strip_suffix('s') {
        values.push(stem.to_owned());
    }
    values
}

fn common_lexicon() -> HashMap<String, String> {
    [
        ("i", "我"), ("you", "你"), ("he", "他"), ("she", "她"), ("it", "它"),
        ("we", "我们"), ("they", "他们"), ("me", "我"), ("him", "他"),
        ("her", "她"), ("them", "他们"), ("my", "我的"), ("your", "你的"),
        ("his", "他的"), ("its", "它的"), ("our", "我们的"), ("their", "他们的"),
        ("this", "这个"), ("that", "那个"), ("these", "这些"), ("those", "那些"),
        ("a", "一个"), ("an", "一个"), ("the", "这个"), ("one", "一"),
        ("two", "二"), ("three", "三"), ("four", "四"), ("five", "五"),
        ("six", "六"), ("seven", "七"), ("eight", "八"), ("nine", "九"),
        ("ten", "十"), ("am", "是"), ("is", "是"), ("are", "是"),
        ("was", "曾是"), ("were", "曾是"), ("be", "是"), ("have", "有"),
        ("has", "有"), ("had", "有"), ("do", "做"), ("does", "做"),
        ("did", "做"), ("can", "能"), ("could", "可以"), ("may", "可以"),
        ("might", "可能"), ("will", "将"), ("would", "会"), ("should", "应该"),
        ("must", "必须"), ("go", "去"), ("come", "来"), ("see", "看见"),
        ("look", "看"), ("know", "知道"), ("like", "喜欢"), ("say", "说"),
        ("read", "读"), ("write", "写"), ("use", "使用"), ("make", "制作"),
        ("take", "拿"), ("give", "给"), ("find", "找到"), ("help", "帮助"),
        ("here", "这里"), ("there", "那里"), ("now", "现在"), ("today", "今天"),
        ("tomorrow", "明天"), ("yesterday", "昨天"), ("again", "再次"),
        ("still", "仍然"), ("very", "很"), ("more", "更多"), ("not", "不"),
        ("all", "所有"), ("some", "一些"), ("any", "任何"), ("every", "每个"),
        ("other", "另一个"), ("in", "在里面"), ("on", "在上面"),
        ("under", "在下面"), ("near", "在附近"), ("inside", "在里面"),
        ("outside", "在外面"), ("at", "在"), ("to", "到"), ("from", "从"),
        ("with", "和"), ("without", "没有"), ("for", "给"), ("of", "的"),
        ("by", "由"), ("about", "关于"), ("across", "穿过"), ("through", "穿过"),
        ("before", "在之前"), ("after", "在之后"), ("between", "在之间"),
        ("among", "在之中"), ("against", "反对"), ("over", "越过"),
        ("and", "和"), ("or", "或"), ("but", "但是"), ("because", "因为"),
        ("if", "如果"), ("when", "当"), ("while", "当"), ("though", "虽然"),
        ("who", "谁"), ("where", "哪里"), ("why", "为什么"), ("how", "怎样"),
        ("yes", "是的"), ("no", "不"), ("people", "人们"), ("person", "人"),
        ("man", "男人"), ("woman", "女人"), ("boy", "男孩"), ("girl", "女孩"),
        ("book", "书"), ("box", "盒子"), ("table", "桌子"), ("room", "房间"),
        ("door", "门"), ("day", "白天"), ("night", "夜晚"), ("morning", "早晨"),
        ("afternoon", "下午"), ("evening", "晚上"), ("food", "食物"),
        ("time", "时间"), ("red", "红色"), ("big", "大的"), ("small", "小的"),
        ("open", "开着的"), ("closed", "关着的"), ("good", "好的"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

fn curated_translation(lesson_id: &str, english: &str) -> Option<&'static str> {
    if lesson_id != "a1-unit-047" {
        return None;
    }
    Some(match english {
        "Tom is a young writer, and Anna is an adult." => "汤姆是一名年轻作家，安娜是一位成年人。",
        "Before a holiday trip, Anna gives Tom some advice about the weather." => "假日旅行前，安娜给了汤姆一些关于天气的建议。",
        "Tom says that his bag is ready, but his warm coat is still at home." => "汤姆说他的包已经准备好了，但保暖外套还在家里。",
        "When they arrive at the station, the sky is dark and the rain begins." => "他们到达车站时，天空变暗，雨也开始下了。",
        "Anna sits on a chair while Tom wants to find a place to buy a coat." => "安娜坐在椅子上，汤姆则想找个地方买外套。",
        "A second adult is waiting near the door with a large bag." => "另一位成年人提着一个大包，在门边等候。",
        "The adult sees Tom and puts a coat on the chair." => "那位成年人看见汤姆，把一件外套放在椅子上。",
        "You may use it during the trip." => "旅行期间你可以穿它。",
        "Tom remembers the advice from Anna and feels sorry because he did not listen." => "汤姆想起安娜的建议，因为自己没有听从而感到后悔。",
        "On the train, the adult tells them that he is also a writer." => "在火车上，那位成年人告诉他们，他也是一名作家。",
        "They talk about the village, the river, and a story with a strange end." => "他们谈到村庄、河流，以及一个结局奇怪的故事。",
        "When the train is at the village, the rain is over and their holiday can begin." => "火车到达村庄时，雨已经停了，他们的假期可以开始了。",
        "That night, Tom writes about the trip, the chair, and the advice that can help him." => "那天晚上，汤姆写下了这次旅行、那把椅子和能够帮助他的建议。",
        "Anna reads the story and says that the new writer can use better advice next time." => "安娜读完故事后说，这位新作家下次可以更好地听取建议。",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_foundation_patterns() {
        let course = CoursePack::embedded().expect("course");
        let guide = TranslationGuide::new(&course);
        let lesson = course.first_lesson().expect("lesson");
        assert_eq!(guide.sentence(lesson, "I am here."), "我在这里。");
        assert_eq!(guide.translate_controlled("This is a book."), "这是一个书。");
    }
}
