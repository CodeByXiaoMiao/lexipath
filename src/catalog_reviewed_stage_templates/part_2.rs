pub(super) fn template(
    word: &str,
) -> Option<(Option<String>, String, String, String)> {
    let lower = word.to_ascii_lowercase();
    let values = match lower.as_str() {
        "hanging" => (Some("n. 悬挂；挂起"), "hanging a picture", "I am hanging a picture.", "You are hanging a picture."),
        "humour" => (None, "like humour", "I like humour.", "Humour is good."),
        "knee" => (None, "a knee", "This is a knee.", "I see a knee."),
        "purpose" => (None, "its purpose", "This is its purpose.", "That is its purpose."),
        "size" => (None, "the right size", "This size is right.", "That size is big."),
        "doubt" => (None, "some doubt", "I have some doubt.", "You have no doubt."),
        "destruction" => (None, "war and destruction", "War and destruction go together.", "Fire and destruction go together."),
        "government" => (None, "the government", "This is the government.", "That is the government."),
        "need" => (None, "a need", "There is a need.", "The need is great."),
        "son" => (None, "has a son", "The man has a son.", "The woman has a son."),
        "interest" => (None, "an interest", "Interest is important.", "This interest is important."),
        "body" => (None, "the body", "The body is important.", "This is a body."),
        "order" => (None, "the right order", "This is the right order.", "That is the right order."),
        "care" => (Some("n. 小心；谨慎"), "take care", "Take care.", "Care is important."),
        "daughter" => (None, "has a daughter", "The woman has a daughter.", "She has a daughter."),
        "amount" => (None, "a small amount", "The amount is small.", "This is a small amount."),
        "trouble" => (None, "trouble", "This is trouble.", "There is trouble here."),
        "brain" => (None, "part of the body", "The brain is part of the body.", "The brain is important."),
        "roll" => (None, "a roll of paper", "This is a roll of paper.", "I have a roll of paper."),
        "lift" => (Some("v. 举起；抬起"), "lift it", "I can lift it.", "You can lift it."),
        "lip" => (None, "the lip", "This is the lip.", "I see the lip."),
        "neck" => (None, "the neck", "This is the neck.", "I see the neck."),
        "paint" => (None, "blue paint", "This paint is blue.", "That paint is red."),
        "nose" => (None, "the nose", "This is the nose.", "I see the nose."),
        "roof" => (None, "the roof", "This is the roof.", "The roof is high."),
        "approval" => (None, "have approval", "I have approval.", "There is approval."),
        "throat" => (None, "the throat", "This is the throat.", "I see the throat."),
        "moon" => (None, "the bright moon", "The moon is bright.", "I see the moon."),
        _ => return None,
    };
    Some((
        values.0.map(str::to_owned),
        values.1.to_owned(),
        values.2.to_owned(),
        values.3.to_owned(),
    ))
}
