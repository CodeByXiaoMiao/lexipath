use crate::phonetics::PhoneticLesson;
use crate::{phonetics_consonants, phonetics_vowels};

pub fn lessons() -> Vec<PhoneticLesson> {
    phonetics_vowels::LESSONS
        .iter()
        .chain(phonetics_consonants::LESSONS.iter())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curriculum_has_fourteen_nonempty_days() {
        let lessons = lessons();
        assert_eq!(lessons.len(), 14);
        assert!(lessons.iter().all(|lesson| !lesson.items.is_empty()));
    }
}
