use crate::course::CoursePack;

use crate::catalog_reviewed_stage_templates::reviewed_stage_template;

pub fn apply_reviewed_stage_templates(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id != "foundation-words" && stage.id != "ogden-850" {
            continue;
        }

        for lesson in &mut stage.lessons {
            let mut changed = false;
            for (index, word) in lesson.new_words.iter_mut().enumerate() {
                let Some((meaning, phrase, first, second)) = reviewed_stage_template(&word.text)
                else {
                    continue;
                };
                changed = true;

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
    use crate::course::{CoursePack, Lesson, Question, Reading, Stage, WordItem};

    #[test]
    fn keeps_question_options_for_lessons_without_reviewed_templates() {
        let expected_options = vec!["I am here.".to_owned(), "You are there.".to_owned()];
        let mut course = CoursePack {
            id: "test".to_owned(),
            title: "test".to_owned(),
            version: 1,
            stages: vec![Stage {
                id: "foundation-words".to_owned(),
                title: "foundation".to_owned(),
                lessons: vec![Lesson {
                    id: "foundation-001".to_owned(),
                    title: "foundation".to_owned(),
                    new_words: vec![WordItem {
                        id: "word-i".to_owned(),
                        text: "I".to_owned(),
                        ipa: "/aɪ/".to_owned(),
                        meaning: "pron. 我".to_owned(),
                        phrase: "I am".to_owned(),
                        example: "I am here.".to_owned(),
                    }],
                    sentences: Vec::new(),
                    reading: Reading {
                        title: "foundation".to_owned(),
                        sentences: Vec::new(),
                        questions: vec![Question {
                            prompt: "test".to_owned(),
                            options: expected_options.clone(),
                            correct_index: 0,
                        }],
                    },
                }],
            }],
        };

        apply_reviewed_stage_templates(&mut course);

        assert_eq!(
            course.stages[0].lessons[0].reading.questions[0].options,
            expected_options
        );
    }
}
