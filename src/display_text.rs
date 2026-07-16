pub fn safe_ipa(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}' => {}
            '\u{ff0f}' | '\u{2215}' | '\u{2044}' => output.push('/'),
            _ if character.is_control() => {}
            _ => output.push(character),
        }
    }
    if output.trim().is_empty() {
        "（音标缺失）".to_owned()
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::safe_ipa;

    #[test]
    fn preserves_ipa_symbols() {
        assert_eq!(safe_ipa("/ˈθɪŋ/"), "/ˈθɪŋ/");
    }

    #[test]
    fn removes_invisible_characters_and_normalizes_slashes() {
        assert_eq!(safe_ipa("\u{feff}／ˈfəʊn⁄"), "/ˈfəʊn/");
    }

    #[test]
    fn shows_a_clear_placeholder_for_missing_ipa() {
        assert_eq!(safe_ipa("\u{200b}\n"), "（音标缺失）");
    }
}
