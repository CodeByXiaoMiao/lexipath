use std::collections::{HashMap, HashSet, VecDeque};

use crate::course::{Lesson, Question, SentenceItem, WordItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    LearnWords,
    Recognition,
    Listening,
    Sentences,
    Reading,
    Comprehension,
    Complete,
}

#[derive(Debug, Clone)]
pub struct AnswerResult {
    pub correct: bool,
    pub remaining: usize,
}

#[derive(Debug, Clone)]
pub struct LearningSession {
    lesson: Lesson,
    phase: Phase,
    learn_index: usize,
    played_word_audio: HashSet<usize>,
    queue: VecDeque<usize>,
    current_audio_played: bool,
    reading_audio_played: bool,
    attempted_items: HashSet<(Phase, usize)>,
    first_attempts: usize,
    first_attempt_errors: usize,
}

impl LearningSession {
    pub fn new(lesson: Lesson) -> Self {
        Self {
            lesson,
            phase: Phase::LearnWords,
            learn_index: 0,
            played_word_audio: HashSet::new(),
            queue: VecDeque::new(),
            current_audio_played: false,
            reading_audio_played: false,
            attempted_items: HashSet::new(),
            first_attempts: 0,
            first_attempt_errors: 0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn lesson(&self) -> &Lesson {
        &self.lesson
    }

    pub fn current_word(&self) -> Option<&WordItem> {
        self.lesson.new_words.get(self.learn_index)
    }

    pub fn mark_word_audio_played(&mut self) {
        self.played_word_audio.insert(self.learn_index);
    }

    pub fn can_advance_word(&self) -> bool {
        self.played_word_audio.contains(&self.learn_index)
    }

    pub fn advance_word(&mut self) -> bool {
        if self.phase != Phase::LearnWords || !self.can_advance_word() {
            return false;
        }

        if self.learn_index + 1 < self.lesson.new_words.len() {
            self.learn_index += 1;
        } else {
            self.start_phase(Phase::Recognition, self.lesson.new_words.len());
        }
        true
    }

    pub fn current_mastery_index(&self) -> Option<usize> {
        self.queue.front().copied()
    }

    pub fn recognition_options(&self) -> Option<(Vec<String>, usize)> {
        let current = self.current_mastery_index()?;
        let mut meaning_counts = HashMap::<&str, usize>::new();
        for word in &self.lesson.new_words {
            *meaning_counts.entry(&word.meaning).or_default() += 1;
        }

        let options = self
            .lesson
            .new_words
            .iter()
            .map(|word| {
                if meaning_counts.get(word.meaning.as_str()).copied().unwrap_or_default() > 1 {
                    format!("{}（{}）", word.meaning, word.phrase)
                } else {
                    word.meaning.clone()
                }
            })
            .collect();

        Some(rotated_options(options, current, current + 1))
    }

    pub fn listening_options(&self) -> Option<(Vec<String>, usize)> {
        let current = self.current_mastery_index()?;
        Some(rotated_options(
            self.lesson
                .new_words
                .iter()
                .map(|word| word.text.clone())
                .collect(),
            current,
            current + 2,
        ))
    }

    pub fn current_sentence(&self) -> Option<&SentenceItem> {
        let index = self.current_mastery_index()?;
        self.lesson.sentences.get(index)
    }

    pub fn sentence_options(&self) -> Option<(Vec<String>, usize)> {
        let current = self.current_mastery_index()?;
        Some(rotated_options(
            self.lesson
                .sentences
                .iter()
                .map(|item| item.meaning.clone())
                .collect(),
            current,
            current + 1,
        ))
    }

    pub fn current_question(&self) -> Option<&Question> {
        let index = self.current_mastery_index()?;
        self.lesson.reading.questions.get(index)
    }

    pub fn mark_current_audio_played(&mut self) {
        self.current_audio_played = true;
    }

    pub fn current_audio_played(&self) -> bool {
        self.current_audio_played
    }

    pub fn answer_current(&mut self, selected_index: usize, correct_index: usize) -> AnswerResult {
        let Some(item) = self.queue.pop_front() else {
            return AnswerResult {
                correct: false,
                remaining: 0,
            };
        };

        let correct = selected_index == correct_index;
        if self.attempted_items.insert((self.phase, item)) {
            self.first_attempts += 1;
            if !correct {
                self.first_attempt_errors += 1;
            }
        }

        if !correct {
            self.queue.push_back(item);
        }
        self.current_audio_played = false;

        if self.queue.is_empty() {
            self.advance_phase_after_mastery();
        }

        AnswerResult {
            correct,
            remaining: self.queue.len(),
        }
    }

    pub fn mark_reading_audio_played(&mut self) {
        self.reading_audio_played = true;
    }

    pub fn reading_audio_played(&self) -> bool {
        self.reading_audio_played
    }

    pub fn finish_reading(&mut self) -> bool {
        if self.phase != Phase::Reading || !self.reading_audio_played {
            return false;
        }
        self.start_phase(Phase::Comprehension, self.lesson.reading.questions.len());
        true
    }

    pub fn first_attempt_accuracy(&self) -> f32 {
        if self.first_attempts == 0 {
            return 1.0;
        }
        (self.first_attempts - self.first_attempt_errors) as f32 / self.first_attempts as f32
    }

    fn start_phase(&mut self, phase: Phase, item_count: usize) {
        self.phase = phase;
        self.queue = (0..item_count).collect();
        self.current_audio_played = false;
    }

    fn advance_phase_after_mastery(&mut self) {
        match self.phase {
            Phase::Recognition => {
                self.start_phase(Phase::Listening, self.lesson.new_words.len())
            }
            Phase::Listening => {
                self.start_phase(Phase::Sentences, self.lesson.sentences.len())
            }
            Phase::Sentences => {
                self.phase = Phase::Reading;
                self.current_audio_played = false;
            }
            Phase::Comprehension => self.phase = Phase::Complete,
            _ => {}
        }
    }
}

fn rotated_options(
    mut options: Vec<String>,
    correct_source_index: usize,
    rotation: usize,
) -> (Vec<String>, usize) {
    if options.is_empty() {
        return (options, 0);
    }
    let len = options.len();
    options.rotate_left(rotation % len);
    let correct_value_index = (correct_source_index + len - (rotation % len)) % len;
    (options, correct_value_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::CoursePack;

    #[test]
    fn a_wrong_answer_stays_in_the_mastery_queue() {
        let course = CoursePack::embedded().expect("course");
        let lesson = course.first_lesson().expect("lesson").clone();
        let word_count = lesson.new_words.len();
        let mut session = LearningSession::new(lesson);

        for _ in 0..word_count {
            session.mark_word_audio_played();
            assert!(session.advance_word());
        }

        assert_eq!(session.phase(), Phase::Recognition);
        let (_, correct) = session.recognition_options().expect("options");
        let wrong = (correct + 1) % word_count;
        let result = session.answer_current(wrong, correct);
        assert!(!result.correct);
        assert_eq!(result.remaining, word_count);
    }

    #[test]
    fn phase_cannot_advance_before_word_audio_is_played() {
        let course = CoursePack::embedded().expect("course");
        let lesson = course.first_lesson().expect("lesson").clone();
        let mut session = LearningSession::new(lesson);
        assert!(!session.advance_word());
        session.mark_word_audio_played();
        assert!(session.advance_word());
    }

    #[test]
    fn duplicate_meanings_are_disambiguated() {
        let course = CoursePack::embedded().expect("course");
        let lesson = course.first_lesson().expect("lesson").clone();
        let word_count = lesson.new_words.len();
        let mut session = LearningSession::new(lesson);

        for _ in 0..word_count {
            session.mark_word_audio_played();
            session.advance_word();
        }

        let (options, _) = session.recognition_options().expect("options");
        let unique = options.iter().collect::<HashSet<_>>();
        assert_eq!(unique.len(), options.len());
    }
}
