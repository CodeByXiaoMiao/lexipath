use crate::course::Stage;

pub fn add_daily_combined_readings(stage: &mut Stage) {
    for pair in stage.lessons.chunks_mut(2) {
        if pair.len() != 2 {
            continue;
        }

        let first_unit_reading = pair[0].reading.sentences.clone();
        pair[1].reading.title = "当日综合零生词阅读".to_owned();
        pair[1]
            .reading
            .sentences
            .extend(first_unit_reading);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{Lesson, Reading, Stage};

    fn lesson(id: &str, sentence: &str) -> Lesson {
        Lesson {
            id: id.to_owned(),
            title: id.to_owned(),
            new_words: Vec::new(),
            sentences: Vec::new(),
            reading: Reading {
                title: "配套零生词阅读".to_owned(),
                sentences: vec![sentence.to_owned()],
                questions: Vec::new(),
            },
        }
    }

    #[test]
    fn second_unit_reuses_first_unit_reading() {
        let mut stage = Stage {
            id: "a1".to_owned(),
            title: "A1".to_owned(),
            lessons: vec![lesson("a", "A."), lesson("b", "B.")],
        };

        add_daily_combined_readings(&mut stage);

        assert_eq!(stage.lessons[1].reading.sentences, ["B.", "A."]);
        assert_eq!(stage.lessons[1].reading.title, "当日综合零生词阅读");
    }
}
