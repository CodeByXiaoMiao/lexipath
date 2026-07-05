use std::collections::{HashSet, VecDeque};

use crate::phonetics::{PhoneticItem, PhoneticLesson};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneticPhase {
    Exposure,
    ListeningTest,
    Complete,
}

#[derive(Debug, Clone)]
pub struct PhoneticSession {
    lesson: PhoneticLesson,
    phase: PhoneticPhase,
    exposure_index: usize,
    played_exposure: HashSet<usize>,
    queue: VecDeque<usize>,
    current_audio_played: bool,
}

impl PhoneticSession {
    pub fn new(lesson: PhoneticLesson) -> Self {
        Self {
            lesson,
            phase: PhoneticPhase::Exposure,
            exposure_index: 0,
            played_exposure: HashSet::new(),
            queue: VecDeque::new(),
            current_audio_played: false,
        }
    }

    pub fn lesson(&self) -> &PhoneticLesson {
        &self.lesson
    }

    pub fn phase(&self) -> PhoneticPhase {
        self.phase
    }

    pub fn current_item(&self) -> Option<&PhoneticItem> {
        match self.phase {
            PhoneticPhase::Exposure => self.lesson.items.get(self.exposure_index),
            PhoneticPhase::ListeningTest => {
                self.queue.front().and_then(|index| self.lesson.items.get(*index))
            }
            PhoneticPhase::Complete => None,
        }
    }

    pub fn mark_audio_played(&mut self) {
        self.current_audio_played = true;
        if self.phase == PhoneticPhase::Exposure {
            self.played_exposure.insert(self.exposure_index);
        }
    }

    pub fn audio_played(&self) -> bool {
        self.current_audio_played
    }

    pub fn advance_exposure(&mut self) -> bool {
        if self.phase != PhoneticPhase::Exposure
            || !self.played_exposure.contains(&self.exposure_index)
        {
            return false;
        }

        self.current_audio_played = false;
        if self.exposure_index + 1 < self.lesson.items.len() {
            self.exposure_index += 1;
        } else {
            self.phase = PhoneticPhase::ListeningTest;
            self.queue = (0..self.lesson.items.len()).collect();
        }
        true
    }

    pub fn test_options(&self) -> Option<(Vec<String>, usize)> {
        let current = *self.queue.front()?;
        let mut options = self
            .lesson
            .items
            .iter()
            .map(|item| item.symbol.to_owned())
            .collect::<Vec<_>>();
        if options.is_empty() {
            return None;
        }
        let rotation = (current + 1) % options.len();
        options.rotate_left(rotation);
        let correct_index = (current + options.len() - rotation) % options.len();
        Some((options, correct_index))
    }

    pub fn answer(&mut self, selected_index: usize, correct_index: usize) -> bool {
        if self.phase != PhoneticPhase::ListeningTest || !self.current_audio_played {
            return false;
        }
        let Some(item) = self.queue.pop_front() else {
            return false;
        };
        let correct = selected_index == correct_index;
        if !correct {
            self.queue.push_back(item);
        }
        self.current_audio_played = false;
        if self.queue.is_empty() {
            self.phase = PhoneticPhase::Complete;
        }
        correct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phonetics_catalog;

    #[test]
    fn exposure_requires_audio() {
        let lesson = phonetics_catalog::lessons().remove(0);
        let mut session = PhoneticSession::new(lesson);
        assert!(!session.advance_exposure());
        session.mark_audio_played();
        assert!(session.advance_exposure());
    }

    #[test]
    fn wrong_test_answer_remains_in_queue() {
        let lesson = phonetics_catalog::lessons().remove(0);
        let count = lesson.items.len();
        let mut session = PhoneticSession::new(lesson);
        for _ in 0..count {
            session.mark_audio_played();
            session.advance_exposure();
        }
        let (_, correct) = session.test_options().expect("options");
        session.mark_audio_played();
        assert!(!session.answer((correct + 1) % count, correct));
        assert_eq!(session.phase(), PhoneticPhase::ListeningTest);
    }
}
