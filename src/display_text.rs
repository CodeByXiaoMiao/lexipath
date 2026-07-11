pub fn safe_ipa(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            'ː' => output.push(':'),
            'ˈ' => output.push('\''),
            'ˌ' => output.push(','),
            'ɪ' | 'i' | 'ɨ' | 'ɯ' => output.push('i'),
            'ʊ' | 'u' | 'ʉ' => output.push('u'),
            'æ' | 'ɑ' | 'ɒ' | 'ɐ' | 'a' => output.push('a'),
            'ɔ' | 'o' | 'ɞ' | 'ɵ' | 'ø' => output.push('o'),
            'ʌ' => output.push('u'),
            'ɜ' | 'ɝ' | 'ɚ' => output.push_str("er"),
            'ə' | 'ә' | 'ɛ' | 'є' | 'ɘ' | 'e' => output.push('e'),
            'θ' => output.push_str("th"),
            'ð' => output.push_str("th"),
            'ʃ' | 'ɕ' | 'ç' => output.push_str("sh"),
            'ʒ' | 'ʑ' => output.push_str("zh"),
            'ŋ' => output.push_str("ng"),
            'ɡ' => output.push('g'),
            'ʤ' | 'ʥ' => output.push('j'),
            'ʧ' | 'ʨ' => output.push_str("ch"),
            'ɹ' | 'ɻ' | 'ʀ' => output.push('r'),
            'ɾ' | 'ʔ' => output.push('t'),
            'ɫ' => output.push('l'),
            'ɲ' => output.push_str("ny"),
            'ʎ' => output.push_str("ly"),
            'œ' => output.push_str("oe"),
            'ɥ' => output.push('w'),
            'β' => output.push('b'),
            _ if character.is_ascii() => output.push(character),
            _ => output.push('?'),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyrillic_lookalikes_do_not_become_question_marks() {
        assert_eq!(safe_ipa("/'fәuldiŋ/"), "/'feulding/");
        assert_eq!(safe_ipa("/hєә/"), "/hee/");
    }
}
