use std::collections::HashSet;

pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        if character.is_ascii_alphabetic() || (character == '\'' && !current.is_empty()) {
            current.push(character.to_ascii_lowercase());
        } else if !current.is_empty() {
            push_token(&mut tokens, &mut current);
        }
    }
    if !current.is_empty() {
        push_token(&mut tokens, &mut current);
    }
    tokens
}

fn push_token(tokens: &mut Vec<String>, current: &mut String) {
    let token = current.trim_matches('\'').to_owned();
    current.clear();
    if token.is_empty() {
        return;
    }
    if let Some(stem) = token.strip_suffix("'s") {
        if !stem.is_empty() {
            tokens.push(stem.to_owned());
        }
    } else {
        tokens.push(token);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphClass {
    Noun,
    Verb,
    Adjective,
    Other,
}

pub fn morph_class(meaning: &str) -> MorphClass {
    let normalized = meaning.trim().to_ascii_lowercase();
    if normalized.starts_with("n.") || normalized.starts_with("pl.") {
        MorphClass::Noun
    } else if normalized.starts_with("v.")
        || normalized.starts_with("vt.")
        || normalized.starts_with("vi.")
    {
        MorphClass::Verb
    } else if normalized.starts_with("adj.") || normalized.starts_with("a.") {
        MorphClass::Adjective
    } else {
        MorphClass::Other
    }
}

pub fn infer_morph_class(
    word: &str,
    meaning: &str,
    phrase: &str,
    example: &str,
) -> MorphClass {
    let declared = morph_class(meaning);
    if declared != MorphClass::Other {
        return declared;
    }
    let word = word.to_ascii_lowercase();
    let phrase = phrase.to_ascii_lowercase();
    let example = example.to_ascii_lowercase();
    if example.starts_with(&format!("{word} "))
        || example.starts_with(&format!("i {word} "))
        || example.starts_with(&format!("you {word} "))
        || example.starts_with(&format!("we {word} "))
        || example.starts_with(&format!("they {word} "))
        || phrase == format!("can {word}")
    {
        MorphClass::Verb
    } else if phrase.starts_with("a ")
        || phrase.starts_with("an ")
        || phrase.starts_with("the ")
        || phrase.starts_with("one ")
    {
        MorphClass::Noun
    } else {
        MorphClass::Other
    }
}

pub fn surface_forms(lemma: &str, class: MorphClass) -> HashSet<String> {
    let lemma = lemma.to_ascii_lowercase();
    let mut forms = HashSet::from([lemma.clone()]);
    if !lemma.chars().all(|character| character.is_ascii_alphabetic()) {
        return forms;
    }

    match class {
        MorphClass::Noun => add_noun_forms(&lemma, &mut forms),
        MorphClass::Verb => add_verb_forms(&lemma, &mut forms),
        MorphClass::Adjective => add_adjective_forms(&lemma, &mut forms),
        MorphClass::Other => {}
    }
    forms
}

pub fn token_matches_entry(surface: &str, lemma: &str, class: MorphClass) -> bool {
    surface_forms(lemma, class).contains(&surface.to_ascii_lowercase())
}

pub fn sequence_count(haystack: &[String], needle: &[String], class: MorphClass) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|window| sequence_matches(window, needle, class))
        .count()
}

pub fn sequence_matches(window: &[String], needle: &[String], class: MorphClass) -> bool {
    if window.len() != needle.len() || needle.is_empty() {
        return false;
    }
    let last = needle.len() - 1;
    window
        .iter()
        .zip(needle)
        .enumerate()
        .all(|(index, (surface, lemma))| {
            if index == last {
                token_matches_entry(surface, lemma, class)
            } else {
                surface == lemma
            }
        })
}

fn add_noun_forms(lemma: &str, forms: &mut HashSet<String>) {
    match lemma {
        "person" => extend(forms, &["people", "persons"]),
        "man" => extend(forms, &["men"]),
        "woman" => extend(forms, &["women"]),
        "child" => extend(forms, &["children"]),
        "foot" => extend(forms, &["feet"]),
        "tooth" => extend(forms, &["teeth"]),
        "mouse" => extend(forms, &["mice"]),
        "goose" => extend(forms, &["geese"]),
        "knife" => extend(forms, &["knives"]),
        "wife" => extend(forms, &["wives"]),
        "life" => extend(forms, &["lives"]),
        "leaf" => extend(forms, &["leaves"]),
        "half" => extend(forms, &["halves"]),
        "shelf" => extend(forms, &["shelves"]),
        "sheep" | "fish" | "species" | "means" => {}
        _ => {
            forms.insert(third_person_or_plural(lemma));
        }
    }
}

fn add_verb_forms(lemma: &str, forms: &mut HashSet<String>) {
    if let Some(irregular) = irregular_verb_forms(lemma) {
        extend(forms, irregular);
        return;
    }

    forms.insert(third_person_or_plural(lemma));
    forms.insert(regular_past(lemma));
    forms.insert(regular_ing(lemma));
}

fn add_adjective_forms(lemma: &str, forms: &mut HashSet<String>) {
    match lemma {
        "good" => extend(forms, &["better", "best"]),
        "bad" => extend(forms, &["worse", "worst"]),
        "far" => extend(forms, &["farther", "farthest", "further", "furthest"]),
        "little" => extend(forms, &["less", "least"]),
        "many" | "much" => extend(forms, &["more", "most"]),
        _ => {}
    }
}

fn irregular_verb_forms(lemma: &str) -> Option<&'static [&'static str]> {
    Some(match lemma {
        "be" => &["am", "is", "are", "was", "were", "been", "being"],
        "have" => &["has", "had", "having"],
        "do" => &["does", "did", "done", "doing"],
        "say" => &["says", "said", "saying"],
        "go" => &["goes", "went", "gone", "going"],
        "come" => &["comes", "came", "coming"],
        "see" => &["sees", "saw", "seen", "seeing"],
        "give" => &["gives", "gave", "given", "giving"],
        "take" => &["takes", "took", "taken", "taking"],
        "make" => &["makes", "made", "making"],
        "get" => &["gets", "got", "gotten", "getting"],
        "find" => &["finds", "found", "finding"],
        "think" => &["thinks", "thought", "thinking"],
        "know" => &["knows", "knew", "known", "knowing"],
        "leave" => &["leaves", "left", "leaving"],
        "feel" => &["feels", "felt", "feeling"],
        "hear" => &["hears", "heard", "hearing"],
        "write" => &["writes", "wrote", "written", "writing"],
        "read" => &["reads", "read", "reading"],
        "speak" => &["speaks", "spoke", "spoken", "speaking"],
        "tell" => &["tells", "told", "telling"],
        "put" => &["puts", "put", "putting"],
        "sit" => &["sits", "sat", "sitting"],
        "stand" => &["stands", "stood", "standing"],
        "run" => &["runs", "ran", "running"],
        "fall" => &["falls", "fell", "fallen", "falling"],
        "forget" => &["forgets", "forgot", "forgotten", "forgetting"],
        "meet" => &["meets", "met", "meeting"],
        "begin" => &["begins", "began", "begun", "beginning"],
        "bring" => &["brings", "brought", "bringing"],
        "buy" => &["buys", "bought", "buying"],
        "catch" => &["catches", "caught", "catching"],
        "choose" => &["chooses", "chose", "chosen", "choosing"],
        "drink" => &["drinks", "drank", "drunk", "drinking"],
        "drive" => &["drives", "drove", "driven", "driving"],
        "eat" => &["eats", "ate", "eaten", "eating"],
        "fly" => &["flies", "flew", "flown", "flying"],
        "grow" => &["grows", "grew", "grown", "growing"],
        "keep" => &["keeps", "kept", "keeping"],
        "lead" => &["leads", "led", "leading"],
        "lose" => &["loses", "lost", "losing"],
        "pay" => &["pays", "paid", "paying"],
        "send" => &["sends", "sent", "sending"],
        "sleep" => &["sleeps", "slept", "sleeping"],
        "teach" => &["teaches", "taught", "teaching"],
        "wear" => &["wears", "wore", "worn", "wearing"],
        "win" => &["wins", "won", "winning"],
        "build" => &["builds", "built", "building"],
        "break" => &["breaks", "broke", "broken", "breaking"],
        "hold" => &["holds", "held", "holding"],
        "understand" => &["understands", "understood", "understanding"],
        _ => return None,
    })
}

fn third_person_or_plural(lemma: &str) -> String {
    if lemma.ends_with('y') && has_consonant_before_final_y(lemma) {
        format!("{}ies", &lemma[..lemma.len() - 1])
    } else if lemma.ends_with('s')
        || lemma.ends_with('x')
        || lemma.ends_with('z')
        || lemma.ends_with("ch")
        || lemma.ends_with("sh")
        || lemma.ends_with('o')
    {
        format!("{lemma}es")
    } else {
        format!("{lemma}s")
    }
}

fn regular_past(lemma: &str) -> String {
    if lemma.ends_with('y') && has_consonant_before_final_y(lemma) {
        format!("{}ied", &lemma[..lemma.len() - 1])
    } else if lemma.ends_with('e') {
        format!("{lemma}d")
    } else if doubles_final_consonant(lemma) {
        let last = lemma.chars().next_back().unwrap_or_default();
        format!("{lemma}{last}ed")
    } else {
        format!("{lemma}ed")
    }
}

fn regular_ing(lemma: &str) -> String {
    if lemma.ends_with("ie") {
        format!("{}ying", &lemma[..lemma.len() - 2])
    } else if lemma.ends_with('e') && !lemma.ends_with("ee") {
        format!("{}ing", &lemma[..lemma.len() - 1])
    } else if doubles_final_consonant(lemma) {
        let last = lemma.chars().next_back().unwrap_or_default();
        format!("{lemma}{last}ing")
    } else {
        format!("{lemma}ing")
    }
}

fn has_consonant_before_final_y(word: &str) -> bool {
    word.as_bytes()
        .get(word.len().saturating_sub(2))
        .map(|byte| !matches!(*byte as char, 'a' | 'e' | 'i' | 'o' | 'u'))
        .unwrap_or(false)
}

fn doubles_final_consonant(word: &str) -> bool {
    matches!(
        word,
        "admit"
            | "beg"
            | "control"
            | "drop"
            | "fit"
            | "grab"
            | "nod"
            | "occur"
            | "permit"
            | "plan"
            | "prefer"
            | "rub"
            | "shop"
            | "slip"
            | "stop"
            | "travel"
    )
}

fn extend(forms: &mut HashSet<String>, values: &[&str]) {
    forms.extend(values.iter().map(|value| (*value).to_owned()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_controlled_regular_verb_forms() {
        let forms = surface_forms("plan", MorphClass::Verb);
        assert!(forms.contains("plans"));
        assert!(forms.contains("planned"));
        assert!(forms.contains("planning"));
        assert!(!forms.contains("planet"));
    }

    #[test]
    fn generates_irregular_verb_forms() {
        let forms = surface_forms("say", MorphClass::Verb);
        assert!(forms.contains("says"));
        assert!(forms.contains("said"));
        assert!(forms.contains("saying"));
    }

    #[test]
    fn counts_inflected_target_uses() {
        let text = vec![
            "tom".to_owned(),
            "plans".to_owned(),
            "a".to_owned(),
            "trip".to_owned(),
        ];
        assert_eq!(
            sequence_count(&text, &["plan".to_owned()], MorphClass::Verb),
            1
        );
    }
}
