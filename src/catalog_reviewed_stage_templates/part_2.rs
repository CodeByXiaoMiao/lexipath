pub(super) fn template(
    word: &str,
) -> Option<(Option<String>, String, String, String)> {
    let lower = word.to_ascii_lowercase();
    let values = match lower.as_str() {
        "hanging" => (None, "hanging pictures", "Hanging pictures is easy.", "Hanging this is hard."),
        "humour" => (None, "like humour", "I like humour.", "Humour can help."),
        "knee" => (None, "my knee", "This is my knee.", "That is your knee."),
        "purpose" => (None, "its purpose", "This is its purpose.", "That is my purpose."),
        "size" => (None, "the right size", "This size is right.", "That size is too big."),
        "doubt" => (None, "some doubt", "I have some doubt.", "You have no doubt."),
        "destruction" => (None, "brings destruction", "War brings destruction.", "Fire can bring destruction."),
        "government" => (None, "the government", "This is the government.", "The government can change."),
        "need" => (None, "a basic need", "Food is a basic need.", "This is a real need."),
        "son" => (None, "my son", "He is my son.", "I know your son."),
        "interest" => (None, "my main interest", "This is my main interest.", "That is your main interest."),
        "body" => (None, "my body", "This is my body.", "The body is strong."),
        "order" => (None, "the right order", "This is the right order.", "That is the right order."),
        "care" => (Some("n. 小心；谨慎"), "take care", "Take care.", "Please take care."),
        "daughter" => (None, "my daughter", "She is my daughter.", "I know your daughter."),
        "amount" => (None, "a large amount", "This is a large amount.", "That is a small amount."),
        "trouble" => (None, "trouble", "This is trouble.", "That means trouble."),
        "brain" => (None, "part of the body", "The brain is part of the body.", "The brain is important."),
        "roll" => (None, "a roll of paper", "This is a roll of paper.", "I have a roll of paper."),
        "lift" => (Some("v. 举起；抬起"), "lift it", "I can lift it.", "You can lift it."),
        "lip" => (None, "my lower lip", "This is my lower lip.", "That is your upper lip."),
        "neck" => (None, "my neck", "This is my neck.", "That is your neck."),
        "paint" => (None, "blue paint", "This paint is blue.", "That paint is red."),
        "nose" => (None, "my nose", "This is my nose.", "That is your nose."),
        "roof" => (None, "the roof", "This is the roof.", "The roof is high."),
        "approval" => (None, "my approval", "This has my approval.", "That has your approval."),
        "throat" => (None, "my throat", "This is my throat.", "That is your throat."),
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
