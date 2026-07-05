use crate::catalog_core_meanings::core_sense_template;
use crate::catalog_function_templates::function_template;
use crate::catalog_semantic_templates::semantic_template;
use crate::catalog_template_overrides::{normalize_display, reviewed_template};
use crate::course::CoursePack;

pub fn apply_reviewed_templates(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id == "foundation-words" {
            continue;
        }

        for lesson in &mut stage.lessons {
            for (index, word) in lesson.new_words.iter_mut().enumerate() {
                let display = normalize_display(&word.text);
                word.text = display.clone();

                let template = if let Some((meaning, phrase, first, second)) =
                    core_sense_template(&display)
                {
                    word.meaning = meaning;
                    Some((phrase, first, second))
                } else {
                    function_template(&display)
                        .or_else(|| reviewed_template(&display))
                        .or_else(|| semantic_template(&display, &word.meaning))
                        .or_else(|| renamed_word_template(&display))
                };

                let Some((phrase, first, second)) = template else {
                    continue;
                };

                word.phrase = phrase;
                word.example = first.clone();

                if let Some(sentence) = lesson.sentences.get_mut(index) {
                    sentence.text = first.clone();
                    sentence.meaning = word.meaning.clone();
                }

                let reading_index = index * 2;
                if let Some(sentence) = lesson.reading.sentences.get_mut(reading_index) {
                    *sentence = first;
                }
                if let Some(sentence) = lesson.reading.sentences.get_mut(reading_index + 1) {
                    *sentence = second;
                }
            }

            let options = lesson
                .new_words
                .iter()
                .map(|word| word.example.clone())
                .collect::<Vec<_>>();
            for (index, question) in lesson.reading.questions.iter_mut().enumerate() {
                question.options = options.clone();
                question.correct_index = index.min(options.len().saturating_sub(1));
            }
        }
    }
}

fn renamed_word_template(word: &str) -> Option<(String, String, String)> {
    let values = match word {
        "core" => ("the core", "This is the core.", "I see the core."),
        "conservative" => (
            "the conservative one",
            "This is the conservative one.",
            "It is the conservative one.",
        ),
        "north" => ("go north", "I can go north.", "You can go north."),
        "god" => ("a god", "This is a god.", "It is a god."),
        "polish" => ("polish this", "I can polish this.", "You can polish it."),
        _ => return None,
    };
    Some((
        values.0.to_owned(),
        values.1.to_owned(),
        values.2.to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_polish::polish_generated_content;
    use crate::course::{CoursePack, Lesson, Reading, Stage, WordItem};

    fn one_word_course(word: &str, meaning: &str) -> CoursePack {
        CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "generated".to_owned(),
                title: "generated".to_owned(),
                lessons: vec![Lesson {
                    id: "unit".to_owned(),
                    title: "unit".to_owned(),
                    new_words: vec![WordItem {
                        id: word.to_owned(),
                        text: word.to_owned(),
                        ipa: "/test/".to_owned(),
                        meaning: meaning.to_owned(),
                        phrase: String::new(),
                        example: String::new(),
                    }],
                    sentences: Vec::new(),
                    reading: Reading {
                        title: String::new(),
                        sentences: Vec::new(),
                        questions: Vec::new(),
                    },
                }],
            }],
        }
    }

    #[test]
    fn reviewed_template_replaces_the_generated_contexts() {
        let mut course = one_word_course("awake", "adj. 醒着的");
        polish_generated_content(&mut course);
        apply_reviewed_templates(&mut course);

        let lesson = &course.stages[0].lessons[0];
        assert_eq!(lesson.new_words[0].example, "I am awake.");
        assert_eq!(lesson.reading.sentences[1], "You are awake.");
    }

    #[test]
    fn core_sense_review_replaces_meaning_and_contexts_together() {
        let mut course = one_word_course("old", "n. 旧事物");
        polish_generated_content(&mut course);
        apply_reviewed_templates(&mut course);

        let word = &course.stages[0].lessons[0].new_words[0];
        assert_eq!(word.meaning, "adj. 老的；旧的");
        assert_eq!(word.example, "It is old.");
    }
}
