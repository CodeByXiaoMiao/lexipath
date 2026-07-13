pub(super) fn template(
    word: &str,
) -> Option<(Option<String>, String, String, String)> {
    let lower = word.to_ascii_lowercase();
    let values = match lower.as_str() {
        "curtain" => (Some("n. 窗帘"), "a blue curtain", "This curtain is blue.", "That curtain is red."),
        "stomach" => (None, "the stomach", "This is the stomach.", "I see the stomach."),
        "bath" => (None, "take a bath", "I take a bath.", "You take a bath."),
        "tongue" => (None, "the tongue", "This is the tongue.", "I see the tongue."),
        "chin" => (None, "the chin", "This is the chin.", "I see the chin."),
        "waiting" => (None, "waiting here", "I am waiting here.", "You are waiting there."),
        "round" => (Some("adj. 圆的"), "is round", "It is round.", "This is round."),
        "attention" => (None, "needs attention", "This needs attention.", "That needs attention."),
        "hope" => (None, "have hope", "I have hope.", "You have hope."),
        "level" => (None, "a high level", "This level is high.", "It is a high level."),
        "knowledge" => (None, "important knowledge", "Knowledge is important.", "Knowledge can help."),
        "current" => (None, "current news", "This is current news.", "Current news can change."),
        "degree" => (None, "to a degree", "It is true to a degree.", "This is true to a degree."),
        "side" => (Some("n. 一边；侧面"), "the other side", "This is the other side.", "That is one side."),
        "weight" => (Some("n. 重量"), "important weight", "Weight is important.", "I know the weight."),
        "earth" => (None, "the round earth", "The earth is round.", "I know about the earth."),
        "scale" => (None, "the scale", "This is the scale.", "I see the scale."),
        "equal" => (Some("adj. 相等的；平等的"), "equal to me", "He is equal to me.", "She is equal to me."),
        "rain" => (None, "see the rain", "I can see the rain.", "You can see the rain."),
        "dust" => (None, "see dust", "I can see dust.", "You can see dust."),
        "steam" => (None, "see steam", "I can see steam.", "You can see steam."),
        "sharp" => (Some("adj. 锋利的；尖的"), "a sharp point", "This point is sharp.", "That point is sharp."),
        "dear" => (Some("adj. 亲爱的；珍爱的"), "dear to me", "She is dear to me.", "He is dear to me."),
        "prose" => (None, "prose", "This is prose.", "I like prose."),
        "sex" => (Some("n. 性别"), "sex on a form", "This form has a line for sex.", "Sex is on this form."),
        "birth" => (None, "a hard birth", "Birth can be hard.", "I know about birth."),
        "opposite" => (None, "the opposite of this", "It is the opposite of this.", "This is the opposite of that."),
        _ => return None,
    };
    Some((
        values.0.map(str::to_owned),
        values.1.to_owned(),
        values.2.to_owned(),
        values.3.to_owned(),
    ))
}
