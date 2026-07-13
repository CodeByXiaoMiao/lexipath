use crate::course::CoursePack;

use crate::catalog_reviewed_stage_templates::reviewed_stage_template;

pub fn apply_reviewed_stage_templates(course: &mut CoursePack) {
    for stage in &mut course.stages {
        if stage.id != "foundation-words" && stage.id != "ogden-850" {
            continue;
        }

        for lesson in &mut stage.lessons {
            for (index, word) in lesson.new_words.iter_mut().enumerate() {
                let Some((meaning, phrase, first, second)) = reviewed_stage_template(&word.text)
                else {
                    continue;
                };

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
