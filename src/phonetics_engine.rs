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
        let option_count = self.lesson.items.len();
        if option_count == 0 {
            return None;
        }

        let correct_index = stable_correct_index(self.lesson.id, current, option_count);
        let mut distractors = (0..option_count)
            .filter(|index| *index != current)
            .collect::<Vec<_>>();
        stable_shuffle(
            &mut distractors,
            stable_seed(self.lesson.id, current, option_count),
        );

        let mut distractors = distractors.into_iter();
        let option_indices = (0..option_count)
            .map(|position| {
                if position == correct_index {
                    current
                } else {
                    distractors
                        .next()
                        .expect("every non-answer position must have a distractor")
                }
            })
            .collect::<Vec<_>>();
        let options = option_indices
            .into_iter()
            .map(|index| self.lesson.items[index].symbol.to_owned())
            .collect();

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

fn stable_correct_index(lesson_id: &str, current: usize, option_count: usize) -> usize {
    let lesson_offset = stable_hash(lesson_id) as usize % option_count;
    (lesson_offset + option_count - current % option_count) % option_count
}

fn stable_seed(lesson_id: &str, current: usize, option_count: usize) -> u64 {
    stable_hash(lesson_id)
        ^ (current as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (option_count as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn stable_shuffle(values: &mut [usize], mut state: u64) {
    for upper in (1..values.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
        let swap_with = state as usize % (upper + 1);
        values.swap(upper, swap_with);
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

    #[test]
    fn test_options_stay_stable_while_the_question_is_visible() {
        let lesson = phonetics_catalog::lessons().remove(0);
        let count = lesson.items.len();
        let mut session = PhoneticSession::new(lesson);
        for _ in 0..count {
            session.mark_audio_played();
            session.advance_exposure();
        }

        assert_eq!(session.test_options(), session.test_options());
    }

    #[test]
    fn correct_answers_cover_every_option_position_in_a_lesson() {
        for lesson in phonetics_catalog::lessons() {
            let count = lesson.items.len();
            let mut session = PhoneticSession::new(lesson);
            for _ in 0..count {
                session.mark_audio_played();
                session.advance_exposure();
            }

            let mut positions = HashSet::new();
            while session.phase() == PhoneticPhase::ListeningTest {
                let (_, correct_index) = session.test_options().expect("options");
                positions.insert(correct_index);
                session.mark_audio_played();
                assert!(session.answer(correct_index, correct_index));
            }

            assert_eq!(positions.len(), count);
            assert!(positions.iter().any(|position| *position + 1 != count));
        }
    }
}
