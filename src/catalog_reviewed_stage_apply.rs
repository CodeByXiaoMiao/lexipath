use crate::catalog_reviewed_a1_templates::reviewed_a1_template;
use crate::catalog_reviewed_a2_templates::reviewed_a2_template;
use crate::catalog_reviewed_b1_templates::reviewed_b1_template;
use crate::catalog_reviewed_stage_templates::reviewed_stage_template;
use crate::course::CoursePack;

pub fn apply_reviewed_stage_templates(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id != "foundation-words"
            && stage.id != "ogden-850"
            && stage.id != "oxford-a1"
            && stage.id != "oxford-a2"
            && stage.id != "oxford-b1"
        {
            continue;
        }

        for lesson in &mut stage.lessons {
            let mut changed = false;
            for (index, word) in lesson.new_words.iter_mut().enumerate() {
                let template = match stage.id.as_str() {
                    "oxford-a1" => reviewed_a1_template(&word.id),
                    "oxford-a2" => reviewed_a2_template(&word.id),
                    "oxford-b1" => reviewed_b1_template(&word.id),
                    _ => reviewed_stage_template(&word.text).map(
                        |(meaning, phrase, first, second)| {
                            (None, meaning, phrase, first, second)
                        },
                    ),
                };
                let Some((display, meaning, phrase, first, second)) = template else {
                    continue;
                };
                changed = true;

                if let Some(display) = display {
                    word.text = display;
                }
                if let Some(meaning) = meaning {
                    word.meaning = meaning;
                }
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

            if !changed {
                continue;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{CoursePack, Lesson, Question, Reading, SentenceItem, Stage, WordItem};

    fn one_word_course(stage_id: &str, word_id: &str, text: &str, example: &str) -> CoursePack {
        CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: stage_id.to_owned(),
                title: "test".to_owned(),
                lessons: vec![Lesson {
                    id: "test-unit".to_owned(),
                    title: "test".to_owned(),
                    new_words: vec![WordItem {
                        id: word_id.to_owned(),
                        text: text.to_owned(),
                        ipa: "/test/".to_owned(),
                        meaning: "test".to_owned(),
                        phrase: text.to_owned(),
                        example: example.to_owned(),
                    }],
                    sentences: vec![SentenceItem {
                        text: example.to_owned(),
                        meaning: "test".to_owned(),
                    }],
                    reading: Reading {
                        title: "test".to_owned(),
                        sentences: vec![example.to_owned(), example.to_owned()],
                        questions: vec![Question {
                            prompt: "test".to_owned(),
                            options: vec![example.to_owned()],
                            correct_index: 0,
                        }],
                    },
                }],
            }],
        }
    }

    #[test]
    fn keeps_question_options_for_lessons_without_reviewed_templates() {
        let mut course = one_word_course("foundation-words", "word-i", "I", "I am here.");
        let expected = course.stages[0].lessons[0].reading.questions[0]
            .options
            .clone();

        apply_reviewed_stage_templates(&mut course);

        assert_eq!(
            course.stages[0].lessons[0].reading.questions[0].options,
            expected
        );
    }

    #[test]
    fn applies_reviewed_a1_templates_by_stable_word_id() {
        let mut course = one_word_course("oxford-a1", "a1-march", "march", "It is march.");

        apply_reviewed_stage_templates(&mut course);

        let lesson = &course.stages[0].lessons[0];
        assert_eq!(lesson.new_words[0].text, "March");
        assert_eq!(lesson.new_words[0].example, "It is March.");
        assert_eq!(lesson.reading.sentences[1], "March is a month.");
        assert_eq!(
            lesson.reading.questions[0].options,
            vec!["It is March.".to_owned()]
        );
    }

    #[test]
    fn applies_reviewed_a2_templates_by_stable_word_id() {
        let mut course =
            one_word_course("oxford-a2", "a2-golf", "golf", "This is a golf.");

        apply_reviewed_stage_templates(&mut course);

        let lesson = &course.stages[0].lessons[0];
        assert_eq!(lesson.new_words[0].example, "I play golf.");
        assert_eq!(lesson.reading.sentences[1], "Golf is popular here.");
        assert_eq!(
            lesson.reading.questions[0].options,
            vec!["I play golf.".to_owned()]
        );
    }

    #[test]
    fn applies_reviewed_b1_templates_by_stable_word_id() {
        let mut course = one_word_course("oxford-b1", "b1-set", "set", "This is a set.");

        apply_reviewed_stage_templates(&mut course);

        let lesson = &course.stages[0].lessons[0];
        assert_eq!(lesson.new_words[0].meaning, "n. 一套；一组");
        assert_eq!(lesson.new_words[0].example, "I bought a set of tools.");
        assert_eq!(lesson.reading.sentences[1], "This set includes six books.");
        assert_eq!(
            lesson.reading.questions[0].options,
            vec!["I bought a set of tools.".to_owned()]
        );
    }
}
