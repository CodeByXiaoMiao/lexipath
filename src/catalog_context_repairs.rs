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
        "since" => repair(
            None,
            "since you are here",
            "Since you are here, I can go.",
            "Since I am here, you can go.",
        ),
        "minute" => repair(
            Some("n. 分钟"),
            "one minute",
            "I have one minute.",
            "You have one minute.",
        ),
        "mine" => repair(
            Some("pron. 我的"),
            "is mine",
            "This book is mine.",
            "The book is mine.",
        ),
        "feeling" => repair(
            Some("n. 感觉"),
            "a feeling",
            "I have a feeling.",
            "You have a feeling.",
        ),
        "driving" => repair(
            Some("n. 驾驶"),
            "driving",
            "I like driving.",
            "You like driving.",
        ),
        "copper" => repair(
            Some("n. 铜"),
            "copper",
            "This is copper.",
            "I have copper.",
        ),
        "iron" => repair(
            Some("n. 铁"),
            "iron",
            "This is iron.",
            "I have iron.",
        ),
        "cent" => repair(
            Some("n. 美分"),
            "a cent",
            "This is a cent.",
            "I have a cent.",
        ),
        "bar" => repair(
            Some("n. 酒吧"),
            "a bar",
            "This is a bar.",
            "I see a bar.",
        ),
        "trip" => repair(
            None,
            "go on a trip",
            "I go on a trip.",
            "You go on a trip.",
        ),
        "human" => repair(
            Some("n. 人"),
            "a human",
            "He is a human.",
            "She is a human.",
        ),
        "individual" => repair(
            Some("n. 个人"),
            "an individual",
            "He is an individual.",
            "She is an individual.",
        ),
        "meet" => repair(
            Some("v. 遇见；会面"),
            "meet me",
            "You can meet me.",
            "I can meet you.",
        ),
        "borrow" => repair(
            Some("v. 借入"),
            "borrow a book",
            "I can borrow a book.",
            "You can borrow a book.",
        ),
        "lend" => repair(
            Some("v. 借出"),
            "lend a book",
            "I can lend a book.",
            "You can lend a book.",
        ),
        "yeah" => repair(
            Some("interj. 是的"),
            "say \"yeah\"",
            "I say \"yeah\".",
            "You say \"yeah\".",
        ),
        "upstairs" => repair(
            Some("adv. 在楼上"),
            "upstairs",
            "I am upstairs.",
            "You are upstairs.",
        ),
        "downstairs" => repair(
            Some("adv. 在楼下"),
            "downstairs",
            "I am downstairs.",
            "You are downstairs.",
        ),
        "apply" => repair(
            Some("v. 申请"),
            "can apply",
            "I can apply.",
            "You can apply.",
        ),
        "solution" => repair(
            Some("n. 解决办法"),
            "a solution",
            "This is a solution.",
            "I have a solution.",
        ),
        "none" => repair(
            Some("pron. 一个也没有"),
            "none",
            "I have none.",
            "You have none.",
        ),
        "recording" => repair(
            Some("n. 录�"),
            "a recording",
            "This is a recording.",
            "I have a recording.",
        ),
        "flying" => repair(
            Some("n. 飞行"),
            "flying",
            "I like flying.",
            "You like flying.",
        ),
        "latest" => repair(
            Some("adj. 最新的"),
            "the latest one",
            "This is the latest one.",
            "It is the latest one.",
        ),
        "global" => repair(
            Some("adj. 全球的"),
            "is global",
            "It is global.",
            "This is global.",
        ),
        "tiny" => repair(
            Some("adj. 极小的"),
            "is tiny",
            "It is tiny.",
            "This is tiny.",
        ),
        "related" => repair(
            Some("adj. 相关的"),
            "related to this",
            "It is related to this.",
            "This is related to it.",
        ),
        "sensible" => repair(
            Some("adj. 明智的"),
            "is sensible",
            "It is sensible.",
            "This is sensible.",
        ),
        "advanced" => repair(
            Some("adj. 高级的；先进的"),
            "is advanced",
            "It is advanced.",
            "This is advanced.",
        ),
        "fighting" => repair(
            Some("n. 打斗"),
            "fighting",
            "I see fighting.",
            "You see fighting.",
        ),
        "indoor" => repair(
            Some("adj. 室内的"),
            "an indoor game",
            "This is an indoor game.",
            "It is an indoor game.",
        ),
        "spoken" => repair(
            Some("adj. 口语的"),
            "a spoken word",
            "This is a spoken word.",
            "It is a spoken word.",
        ),
        "spicy" => repair(
            Some("adj. 辛辣的"),
            "is spicy",
            "It is spicy.",
            "This is spicy.",
        ),
        "located" => repair(
            Some("adj. 位于"),
            "located here",
            "It is located here.",
            "This is located here.",
        ),
        "long term" => repair(
            Some("adj. 长期的"),
            "is long term",
            "This plan is long term.",
            "The plan is long term.",
        ),
        "creature" => repair(
            Some("n. 生物"),
            "a creature",
            "This is a creature.",
            "I see a creature.",
        ),
        "critical" => repair(
            Some("adj. 关键的"),
            "is critical",
            "It is critical.",
            "This is critical.",
        ),
        "vital" => repair(
            Some("adj. 至关重要的"),
            "is vital",
            "It is vital.",
            "This is vital.",
        ),
        "secure" => repair(
            Some("adj. 安全的"),
            "is secure",
            "It is secure.",
            "This is secure.",
        ),
        "severe" => repair(
            Some("adj. 严重的"),
            "is severe",
            "It is severe.",
            "This is severe.",
        ),
        "intense" => repair(
            Some("adj. 强烈的"),
            "is intense",
            "It is intense.",
            "This is intense.",
        ),
        "virtual" => repair(
            Some("adj. 虚拟的"),
            "is virtual",
            "It is virtual.",
            "This is virtual.",
        ),
        "minimum" => repair(
            Some("n. 最低限度"),
            "the minimum",
            "This is the minimum.",
            "It is the minimum.",
        ),
        "outer" => repair(
            Some("adj. 外部的"),
            "the outer part",
            "This is the outer part.",
            "It is the outer part.",
        ),
        "solar" => repair(
            Some("adj. 太阳能的"),
            "solar power",
            "This is solar power.",
            "It is solar power.",
        ),
        "daily" => repair(
            Some("adj. 每日的"),
            "daily work",
            "This is daily work.",
            "It is daily work.",
        ),
        "financial" => repair(
            Some("adj. 财务的"),
            "financial work",
            "This is financial work.",
            "It is financial work.",
        ),
        "previous" => repair(
            Some("adj. 先前的"),
            "the previous one",
            "This is the previous one.",
            "It is the previous one.",
        ),
        "primary" => repair(
            Some("adj. 主要的"),
            "the primary reason",
            "This is the primary reason.",
            "It is the primary reason.",
        ),
        "nuclear" => repair(
            Some("adj. 核能的"),
            "nuclear power",
            "This is nuclear power.",
            "It is nuclear power.",
        ),
        "technical" => repair(
            Some("adj. 技术的"),
            "technical work",
            "This is technical work.",
            "It is technical work.",
        ),
        "academic" => repair(
            Some("adj. 学术的"),
            "academic work",
            "This is academic work.",
            "It is academic work.",
        ),
        "mental" => repair(
            Some("adj. 心理的"),
            "mental health",
            "This is mental health.",
            "I know about mental health.",
        ),
        "educational" => repair(
            Some("adj. 教育的"),
            "educational work",
            "This is educational work.",
            "It is educational work.",
        ),
        "scientific" => repair(
            Some("adj. 科学的"),
            "scientific work",
            "This is scientific work.",
            "It is scientific work.",
        ),
        "secondary" => repair(
            Some("adj. 中学的；次要的"),
            "a secondary school",
            "This is a secondary school.",
            "It is a secondary school.",
        ),
        "written" => repair(
            Some("adj. 书面的"),
            "written work",
            "This is written work.",
            "It is written work.",
        ),
        "remote" => repair(
            Some("adj. 偏远的"),
            "a remote area",
            "This is a remote area.",
            "It is a remote area.",
        ),
        "aged" => repair(
            Some("adj. 年老的"),
            "an aged person",
            "He is an aged person.",
            "She is an aged person.",
        ),
        "repeated" => repair(
            Some("adj. 重复的"),
            "repeated work",
            "This is repeated work.",
            "It is repeated work.",
        ),
        "additional" => repair(
            Some("adj. 额外的"),
            "additional work",
            "This is additional work.",
            "It is additional work.",
        ),
        "rural" => repair(
            Some("adj. 乡村的"),
            "a rural area",
            "This is a rural area.",
            "It is a rural area.",
        ),
        "detailed" => repair(
            Some("adj. 详细的"),
            "a detailed plan",
            "This is a detailed plan.",
            "It is a detailed plan.",
        ),
        "external" => repair(
            Some("adj. 外部的"),
            "the external part",
            "This is the external part.",
            "It is the external part.",
        ),
        "visual" => repair(
            Some("adj. 视觉的"),
            "visual art",
            "This is visual art.",
            "It is visual art.",
        ),
        "artistic" => repair(
            Some("adj. 艺术的"),
            "artistic work",
            "This is artistic work.",
            "It is artistic work.",
        ),
        "associated" => repair(
            Some("adj. 相关的"),
            "associated with this",
            "It is associated with this.",
            "This is associated with it.",
        ),
        "intended" => repair(
            Some("adj. 预定的；有意的"),
            "intended for you",
            "It is intended for you.",
            "This is intended for me.",
        ),
        "rise" => repair(
            Some("v. 上升；升起"),
            "can rise",
            "It can rise.",
            "This can rise.",
        ),
        "soft" => repair(
            None,
            "is soft",
            "It is soft.",
            "This is soft.",
        ),
        "involve" => repair(
            None,
            "involve work",
            "This can involve work.",
            "It can involve work.",
        ),
        "surround" => repair(
            None,
            "surround this",
            "They can surround this.",
            "We can surround it.",
        ),
        _ => return None,
    };
    Some(repair)
}

const fn repair(
    meaning: Option<&'static str>,
    phrase: &'static str,
    first: &'static str,
    second: &'static str,
) -> ContextRepair {
    ContextRepair {
        meaning,
        phrase,
        first,
        second,
    }
}
