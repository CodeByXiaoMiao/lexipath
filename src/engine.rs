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

pub struct QuestionLookup(Option<Question>);

impl QuestionLookup {
    pub fn cloned(self) -> Option<Question> {
        self.0
    }
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
        let phase = if lesson.is_stage_assessment() {
            Phase::Reading
        } else {
            Phase::LearnWords
        };
        Self {
            lesson,
            phase,
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
        let mut counts = HashMap::<&str, usize>::new();
        for word in &self.lesson.new_words {
            *counts.entry(&word.meaning).or_default() += 1;
        }
        let options = self
            .lesson
            .new_words
            .iter()
            .map(|word| {
                if counts.get(word.meaning.as_str()).copied().unwrap_or_default() > 1 {
                    format!("{}（{}）", word.meaning, word.phrase)
                } else {
                    word.meaning.clone()
                }
            })
            .collect();
        Some(stable_options(
            options,
            current,
            option_seed(&self.lesson.id, "recognition", current),
        ))
    }

    pub fn listening_options(&self) -> Option<(Vec<String>, usize)> {
        let current = self.current_mastery_index()?;
        Some(stable_options(
            self.lesson
                .new_words
                .iter()
                .map(|word| word.text.clone())
                .collect(),
            current,
            option_seed(&self.lesson.id, "listening", current),
        ))
    }

    pub fn current_sentence(&self) -> Option<&SentenceItem> {
        let index = self.current_mastery_index()?;
        self.lesson.sentences.get(index)
    }

    pub fn sentence_options(&self) -> Option<(Vec<String>, usize)> {
        let current = self.current_mastery_index()?;
        Some(stable_options(
            self.lesson
                .sentences
                .iter()
                .map(|item| item.meaning.clone())
                .collect(),
            current,
            option_seed(&self.lesson.id, "sentences", current),
        ))
    }

    pub fn current_question(&self) -> QuestionLookup {
        let Some(index) = self.current_mastery_index() else {
            return QuestionLookup(None);
        };
        if let Some(question) = self.lesson.reading.questions.get(index) {
            return QuestionLookup(Some(stable_question(
                question.clone(),
                option_seed(&self.lesson.id, "comprehension", index),
            )));
        }
        let Some(sentence) = self.lesson.sentences.get(index) else {
            return QuestionLookup(None);
        };
        QuestionLookup(Some(stable_question(
            Question {
                prompt: format!("请选择与中文含义对应的句子：{}", sentence.meaning),
                options: self
                    .lesson
                    .sentences
                    .iter()
                    .map(|item| item.text.clone())
                    .collect(),
                correct_index: index,
            },
            option_seed(&self.lesson.id, "comprehension-fallback", index),
        )))
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
        let count = if self.lesson.reading.questions.is_empty() {
            self.lesson.sentences.len()
        } else {
            self.lesson.reading.questions.len()
        };
        self.start_phase(Phase::Comprehension, count);
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

fn stable_question(mut question: Question, seed: u64) -> Question {
    let (options, correct_index) = stable_options(question.options, question.correct_index, seed);
    question.options = options;
    question.correct_index = correct_index;
    question
}

fn stable_options(
    options: Vec<String>,
    correct_source_index: usize,
    mut state: u64,
) -> (Vec<String>, usize) {
    if options.is_empty() {
        return (options, 0);
    }
    let mut indexed = options.into_iter().enumerate().collect::<Vec<_>>();
    for upper in (1..indexed.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
        let swap_with = state as usize % (upper + 1);
        indexed.swap(upper, swap_with);
    }
    let correct_index = indexed
        .iter()
        .position(|(source, _)| *source == correct_source_index)
        .unwrap_or(0);
    let options = indexed.into_iter().map(|(_, value)| value).collect();
    (options, correct_index)
}

fn option_seed(lesson_id: &str, phase: &str, item_index: usize) -> u64 {
    stable_hash(lesson_id)
        ^ stable_hash(phase).rotate_left(17)
        ^ (item_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::{CoursePack, Reading};

    fn recognition_session() -> LearningSession {
        let course = CoursePack::embedded().expect("course");
        let lesson = course.first_lesson().expect("lesson").clone();
        let count = lesson.new_words.len();
        let mut session = LearningSession::new(lesson);
        for _ in 0..count {
            session.mark_word_audio_played();
            session.advance_word();
        }
        session
    }

    #[test]
    fn a_wrong_answer_stays_in_the_mastery_queue() {
        let mut session = recognition_session();
        let count = session.lesson().new_words.len();
        let (_, correct) = session.recognition_options().expect("options");
        let wrong = (correct + 1) % count;
        let result = session.answer_current(wrong, correct);
        assert!(!result.correct);
        assert_eq!(result.remaining, count);
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
        let session = recognition_session();
        let (options, _) = session.recognition_options().expect("options");
        let unique = options.iter().collect::<HashSet<_>>();
        assert_eq!(unique.len(), options.len());
    }

    #[test]
    fn stage_assessment_starts_at_reading() {
        let lesson = Lesson {
            id: "stage-final-ogden-850".to_owned(),
            title: "总结".to_owned(),
            new_words: Vec::new(),
            sentences: Vec::new(),
            reading: Reading {
                title: "总结阅读".to_owned(),
                sentences: vec!["I am here.".to_owned()],
                questions: vec![Question {
                    prompt: "问题".to_owned(),
                    options: vec!["I am here.".to_owned(), "You are here.".to_owned()],
                    correct_index: 0,
                }],
            },
        };
        assert_eq!(LearningSession::new(lesson).phase(), Phase::Reading);
    }
}
