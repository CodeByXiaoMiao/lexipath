mod part_1;
mod part_2;
mod part_3;

pub fn reviewed_stage_template(
    word: &str,
) -> Option<(Option<String>, String, String, String)> {
    part_1::template(word)
        .or_else(|| part_2::template(word))
        .or_else(|| part_3::template(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_unidiomatic_foundation_and_ogden_examples() {
        assert_eq!(reviewed_stage_template("rain").expect("rain template").2, "I can see the rain.");
        assert_eq!(reviewed_stage_template("earth").expect("earth template").2, "The earth is round.");
        assert_eq!(reviewed_stage_template("prose").expect("prose template").2, "This is prose.");
        assert_eq!(reviewed_stage_template("son").expect("son template").2, "He is my son.");
        assert_eq!(reviewed_stage_template("trouble").expect("trouble template").2, "This is trouble.");
        assert_eq!(reviewed_stage_template("waiting").expect("waiting template").2, "I am waiting here.");
        assert_eq!(reviewed_stage_template("damage").expect("damage template").2, "I can see the damage.");
        assert_eq!(
            reviewed_stage_template("round")
                .expect("round template")
                .0
                .as_deref(),
            Some("adj. 圆的")
        );
    }
}
