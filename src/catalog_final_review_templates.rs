pub fn final_review_template(word: &str) -> Option<(Option<String>, String, String, String)> {
    let lower = word.to_ascii_lowercase();

    if ABSTRACT_TOPIC_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            None,
            &format!("know about {lower}"),
            &format!("I know about {lower}."),
            &format!("You know about {lower}."),
        ));
    }
    if MASS_HAVE_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            None,
            &lower,
            &format!("I have {lower}."),
            &format!("You have {lower}."),
        ));
    }
    if ING_ACTIVITY_NOUNS.contains(&lower.as_str()) {
        return Some(tuple(
            None,
            &lower,
            &format!("I like {lower}."),
            &format!("You like {lower}."),
        ));
    }
    if PERSON_NOUNS.contains(&lower.as_str()) {
        let article = article(&lower);
        return Some(tuple(
            None,
            &format!("{article} {lower}"),
            &format!("He is {article} {lower}."),
            &format!("She is {article} {lower}."),
        ));
    }
    if PERSON_ADJECTIVES.contains(&lower.as_str()) {
        return Some(tuple(
            None,
            &format!("be {lower}"),
            &format!("He is {lower}."),
            &format!("She is {lower}."),
        ));
    }

    let values = match lower.as_str() {
        // Function and early Ogden words where the generated phrase was too bare.
        "before" => (None, "before you", "I go before you.", "You come before me."),
        "ever" => (None, "ever go", "Do you ever go?", "Do I ever come?"),
        "please" => (None, "please come here", "Please come here.", "Please go there."),
        "anything" => (None, "see anything", "Do you see anything?", "I do not see anything."),
        "either" => (None, "either book", "I can use either book.", "You can use either book."),
        "none" => (None, "no one", "No one is here.", "I see no one."),
        "neither" => (None, "neither book", "Neither book is here.", "I see neither book."),
        "since" => (None, "since you are here", "I stay since you are here.", "You stay since I am here."),
        "per" => (None, "one per person", "One book per person.", "One day per person."),
        "despite" => (None, "despite this", "Despite this, I go.", "Despite this, you come."),
        "nor" => (None, "nor do you", "I do not go, nor do you.", "You do not come, nor do I."),
        "unlike" => (None, "unlike that", "It is unlike that.", "This is unlike that."),

        // Meaning repairs and common POS repairs.
        "lead" => (Some("vt. 带领；引导"), "lead me", "You can lead me.", "I can lead you."),
        "fall" => (Some("n. 秋天；秋季"), "in the fall", "I go in the fall.", "You come in the fall."),
        "dead" => (Some("adj. 死的；无生命的"), "be dead", "He is dead.", "It is dead."),
        "burn" => (Some("v. 燃烧；烧伤"), "can burn", "It can burn.", "This can burn."),
        "shut" => (Some("adj. 关闭的；v. 关闭"), "is shut", "It is shut.", "This is shut."),
        "revise" => (Some("v. 修改；复习"), "revise it", "I can revise it.", "You can revise it."),
        "fry" => (Some("v. 油炸；煎"), "fry food", "I can fry food.", "You can fry food."),
        "settle" => (Some("v. 定居；解决"), "settle it", "I can settle it.", "You can settle it."),
        "impress" => (Some("v. 给……深刻印象"), "impress me", "It can impress me.", "This can impress you."),
        "awful" => (Some("adj. 糟糕的；可怕的"), "be awful", "It is awful.", "This is awful."),
        "download" => (Some("v. 下载"), "download it", "I can download it.", "You can download it."),
        "continent" => (Some("n. 大陆；洲"), "a continent", "This is a continent.", "I see a continent."),
        "funding" => (Some("n. 资金；资助"), "funding", "I know about funding.", "You know about funding."),
        "somewhat" => (Some("adv. 有点；稍微"), "somewhat different", "It is somewhat different.", "This is somewhat different."),
        "main" => (Some("adj. 主要的"), "the main part", "This is the main part.", "It is the main part."),
        "select" => (Some("adj. 精选的；优等的"), "a select group", "This is a select group.", "It is a select group."),
        "folding" => (Some("adj. 可折叠的"), "a folding chair", "This is a folding chair.", "It is a folding chair."),
        "prime" => (Some("adj. 首要的；主要的"), "prime time", "This is prime time.", "It is prime time."),
        "multiple" => (Some("adj. 多个的；多重的"), "multiple books", "I see multiple books.", "You have multiple books."),
        "contemporary" => (Some("adj. 当代的"), "a contemporary book", "This is a contemporary book.", "I read a contemporary book."),
        "tough" => (Some("adj. 艰难的；坚强的"), "be tough", "It is tough.", "This is tough."),
        "lean" => (Some("v. 倾斜；依靠"), "lean on this", "I can lean on this.", "You can lean on it."),

        // Nouns needing a determiner or a clearer context.
        "minute" => (None, "one minute", "This is one minute.", "I wait one minute."),
        "future" => (None, "the future", "This is the future.", "I know about the future."),
        "past" => (None, "the past", "This is the past.", "I know about the past."),
        "sound" => (None, "a sound", "I hear a sound.", "You hear a sound."),
        "sex" => (None, "sex", "I know about sex.", "You know about sex."),
        "wind" => (None, "the wind", "This is the wind.", "I feel the wind."),
        "sky" => (None, "the sky", "This is the sky.", "I see the sky."),
        "plane" => (None, "a plane", "This is a plane.", "I see a plane."),
        "wood" => (None, "wood", "I have wood.", "You have wood."),
        "cry" => (None, "a cry", "I hear a cry.", "You hear a cry."),
        "birth" => (None, "birth", "I know about birth.", "You know about birth."),
        "snow" => (None, "snow", "I see snow.", "You see snow."),
        "steel" => (None, "steel", "I have steel.", "You have steel."),
        "opposite" => (None, "opposite this", "It is opposite this.", "This is opposite that."),
        "steam" => (None, "steam", "I see steam.", "You see steam."),
        "trousers" => (None, "trousers", "These are trousers.", "I see trousers."),
        "chalk" => (None, "chalk", "I have chalk.", "You have chalk."),
        "centre" => (None, "the centre", "This is the centre.", "I see the centre."),
        "neighbour" => (Some("n. 邻居"), "a neighbour", "He is a neighbour.", "She is a neighbour."),
        "programme" => (None, "a TV programme", "This is a TV programme.", "I see a TV programme."),
        "team" => (None, "a team", "This team is here.", "We are a team."),
        "parent" => (None, "a parent", "He is a parent.", "She is a parent."),
        "period" => (None, "a time period", "This is a time period.", "I know this period."),
        "cost" => (None, "the cost", "This is the cost.", "I know the cost."),
        "holiday" => (None, "a holiday", "Today is a holiday.", "I have a holiday."),
        "trip" => (None, "a trip", "I have a trip.", "You have a trip."),
        "somebody" => (None, "somebody", "Somebody is here.", "I see somebody."),
        "tourist" => (None, "a tourist", "He is a tourist.", "She is a tourist."),
        "routine" => (None, "a routine", "I have a routine.", "You have a routine."),
        "policeman" => (None, "a policeman", "He is a policeman.", "I see a policeman."),
        "downstairs" => (None, "downstairs", "I am downstairs.", "You are downstairs."),
        "reach" => (Some("v. 到达；够到"), "reach it", "I can reach it.", "You can reach it."),
        "sir" => (None, "sir", "Sir, please come here.", "Sir, please go there."),
        "population" => (None, "the population", "The population is large.", "I know the population."),
        "rise" => (None, "a rise", "This is a rise in price.", "I see a rise."),
        "author" => (None, "an author", "He is an author.", "She is an author."),
        "race" => (None, "a race", "This is a race.", "I see a race."),
        "camp" => (None, "a camp", "This is a camp.", "I go to a camp."),
        "route" => (None, "a route", "This is a route.", "I know the route."),
        "hide" => (Some("v. 躲藏；隐藏"), "hide it", "I can hide it.", "You can hide it."),
        "twin" => (None, "a twin", "He is a twin.", "She is a twin."),
        "blank" => (None, "a blank space", "This is a blank space.", "I see a blank space."),
        "chat" => (None, "a chat", "I have a chat.", "You have a chat."),
        "underground" => (None, "the underground", "This is the underground.", "I use the underground."),
        "staff" => (None, "the staff", "The staff are here.", "I see the staff."),
        "effort" => (None, "an effort", "I make an effort.", "You make an effort."),
        "claim" => (None, "a claim", "I make a claim.", "You make a claim."),
        "shoot" => (Some("v. 射击；拍摄"), "shoot it", "I can shoot it.", "You can shoot it."),
        "departure" => (None, "departure time", "This is departure time.", "I know the departure time."),
        "hurry" => (None, "in a hurry", "I am in a hurry.", "You are in a hurry."),
        "scan" => (Some("v. 扫描；浏览"), "scan it", "I can scan it.", "You can scan it."),
        "executive" => (None, "an executive", "He is an executive.", "She is an executive."),
        "assessment" => (None, "an assessment", "This is an assessment of work.", "I know about assessment."),
        "resident" => (None, "a resident", "He is a resident.", "She is a resident."),
        "objective" => (None, "an objective", "I have an objective.", "You have an objective."),
        "wage" => (None, "a wage", "This is the wage.", "I know the wage."),
        "minority" => (None, "a minority", "This is a minority group.", "I know about a minority."),
        "crew" => (None, "the crew", "The crew is here.", "I see the crew."),
        "frequency" => (None, "a frequency", "This is a high frequency.", "I know the frequency."),
        "fellow" => (None, "a fellow", "He is a fellow.", "I see a fellow."),
        "giant" => (None, "a giant", "He is a giant.", "I see a giant."),
        "sink" => (None, "a sink", "This is a kitchen sink.", "I see a sink."),
        "handle" => (None, "a handle", "This is a door handle.", "I see a handle."),
        "master" => (None, "a master", "He is a master.", "She is a master."),
        "league" => (None, "a league", "This is a sports league.", "I know this league."),
        "dozen" => (None, "a dozen books", "I have a dozen books.", "You have a dozen books."),
        "couple" => (None, "a couple", "They are a couple.", "I see a couple."),

        // Adjectives and adverbs needing better collocation.
        "natural" => (None, "be natural", "It is natural.", "This is natural."),
        "happy" => (None, "be happy", "I am happy.", "You are happy."),
        "warm" => (None, "be warm", "It is warm.", "This is warm."),
        "soft" => (None, "a soft book", "This is a soft book.", "It is soft."),
        "regular" => (None, "a regular day", "This is a regular day.", "It is regular."),
        "dependent" => (None, "be dependent", "He is dependent.", "She is dependent."),
        "frequent" => (None, "frequent use", "This is frequent use.", "I know about frequent use."),
        "best" => (None, "the best book", "It is the best book.", "This is the best book."),
        "mobile" => (None, "a mobile phone", "This is a mobile phone.", "It is mobile."),
        "lazy" => (None, "be lazy", "He is lazy.", "She is lazy."),
        "usual" => (None, "the usual one", "This is the usual one.", "It is the usual one."),
        "industrial" => (None, "an industrial area", "This is an industrial area.", "It is industrial."),
        "regional" => (None, "a regional area", "This is a regional area.", "It is regional."),
        "relevant" => (None, "relevant to this", "It is relevant to this.", "This is relevant."),
        "anxious" => (None, "be anxious", "He is anxious.", "She is anxious."),

        // Verbs needing objects or safer frames.
        "believe" => (None, "believe it", "I can believe it.", "You can believe it."),
        "want" => (Some("v. 想要"), "want it", "I want it.", "You want it."),
        "carry" => (Some("v. 携带；搬运"), "carry it", "I can carry it.", "You can carry it."),
        "wait" => (Some("v. 等待"), "wait here", "I wait here.", "You wait there."),
        "die" => (None, "can die", "People can die.", "It can die."),
        "drive" => (Some("v. 驾驶；驱动"), "drive it", "I can drive it.", "You can drive it."),
        "wear" => (Some("v. 穿；戴"), "wear it", "I wear it.", "You wear it."),
        "close" => (Some("v. 关闭"), "close it", "I can close it.", "You can close it."),
        "spend" => (None, "spend money", "I spend money on books.", "You spend money on food."),
        "collect" => (None, "collect it", "I can collect it.", "You can collect it."),
        "prevent" => (None, "prevent it", "I can prevent it.", "You can prevent it."),
        "determine" => (None, "determine it", "I can determine it.", "You can determine it."),
        "extend" => (None, "extend it", "I can extend it.", "You can extend it."),
        "deny" => (None, "deny it", "I can deny it.", "You can deny it."),
        "alter" => (None, "alter it", "I can alter it.", "You can alter it."),
        "assume" => (None, "assume this", "I assume this.", "You assume it."),
        "dig" => (Some("v. 挖；掘"), "dig here", "I can dig here.", "You can dig there."),
        "quit" => (None, "quit it", "I can quit it.", "You can quit it."),
        "pray" => (None, "pray every day", "I pray every day.", "You pray every day."),
        "confuse" => (None, "confuse me", "It can confuse me.", "This can confuse you."),
        "summarize" => (None, "summarize it", "I can summarize it.", "You can summarize it."),
        "dislike" => (Some("v. 不喜欢"), "dislike it", "I dislike it.", "You dislike it."),
        "grab" => (Some("v. 抓住"), "grab it", "I can grab it.", "You can grab it."),
        "offend" => (None, "offend him", "It can offend him.", "This can offend her."),
        "emerge" => (None, "can emerge", "A problem can emerge.", "This can emerge."),
        "occur" => (None, "can occur", "A change can occur.", "This can occur."),
        "kill" => (Some("v. 杀死；消灭"), "kill it", "It can kill it.", "This can kill it."),
        "save" => (Some("v. 保存；拯救"), "save it", "I can save it.", "You can save it."),
        "gather" => (Some("v. 聚集；收集"), "gather here", "People gather here.", "We gather here."),
        "retire" => (Some("v. 退休"), "retire soon", "I can retire soon.", "You can retire soon."),

        // Sports, subjects, and activities.
        "golf" => (None, "play golf", "I play golf.", "You play golf."),
        "flu" => (None, "the flu", "I have the flu.", "You have the flu."),
        "staff" => (None, "the staff", "The staff are here.", "I see the staff."),
        "folk" => (None, "folk music", "This is folk music.", "I know folk music."),
        "digestion" => (None, "digestion", "I know about digestion.", "You know about digestion."),
        "hanging" => (None, "hanging", "I know about hanging.", "You know about hanging."),
        "spelling" => (None, "spelling", "I know about spelling.", "You know about spelling."),
        "swim" => (Some("v. 游泳"), "swim", "I can swim.", "You can swim."),
        _ => return None,
    };
    Some(tuple(
        values.0.map(str::to_owned),
        values.1,
        values.2,
        values.3,
    ))
}

const ABSTRACT_TOPIC_NOUNS: &[&str] = &[
    "damage", "distance", "danger", "existence", "hate", "self", "living", "waiting",
    "disgust", "advice", "ability", "difference", "culture", "understanding", "climate",
    "unemployment", "childhood", "spending", "entertainment", "qualification",
    "excitement", "photography", "breathing", "printing", "regard", "opposition",
    "trust", "conduct", "protection", "consideration", "faith", "hell", "moral",
    "discipline", "finance", "blame", "heaven", "enthusiasm", "imagination",
    "psychology", "privacy", "sympathy", "popularity", "economy", "production",
    "accommodation", "countryside", "accommodation", "funding", "respect", "fiction",
    "permission", "transport", "security", "safety", "importance", "independence",
];

const MASS_HAVE_NOUNS: &[&str] = &[
    "cheese", "wine", "sand", "silver", "dust", "soup", "cotton", "cash", "mail",
    "jewellery", "fuel", "flour", "software", "countryside", "rain", "rubbish",
];

const ING_ACTIVITY_NOUNS: &[&str] = &[
    "shopping", "swimming", "running", "farming", "camping", "racing", "hunting",
    "shooting",
];

const PERSON_NOUNS: &[&str] = &[
    "executive", "master", "resident", "suspect", "fellow", "tourist", "policeman",
    "author", "parent", "neighbour",
];

const PERSON_ADJECTIVES: &[&str] = &[
    "happy", "dead", "lazy", "anxious", "dependent", "female", "depressed", "grateful",
];

fn article(word: &str) -> &'static str {
    if matches!(word, "executive" | "author") {
        return "an";
    }
    "a"
}

fn tuple(
    meaning: Option<String>,
    phrase: &str,
    first: &str,
    second: &str,
) -> Option<(Option<String>, String, String, String)> {
    Some((
        meaning,
        phrase.to_owned(),
        first.to_owned(),
        second.to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repairs_bad_dictionary_senses() {
        let template = final_review_template("download").expect("template");
        assert_eq!(template.0.as_deref(), Some("v. 下载"));
        assert_eq!(template.2, "I can download it.");
    }

    #[test]
    fn keeps_abstract_nouns_out_of_a_this_is_frame() {
        let template = final_review_template("privacy").expect("template");
        assert_eq!(template.2, "I know about privacy.");
    }

    #[test]
    fn repairs_transitive_verbs_with_objects() {
        let template = final_review_template("summarize").expect("template");
        assert_eq!(template.2, "I can summarize it.");
    }
}
