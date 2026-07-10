# Course rules

LexiPath has one fixed learning path. Users do not edit lesson order, daily volume, pass thresholds, or review rules.

## Mandatory unit sequence

1. Learn every new word and play its English pronunciation.
2. Pass word-meaning recognition.
3. Pass listening recognition.
4. Pass paired sentence training.
5. Read the controlled text and play the complete English reading.
6. Pass reading comprehension.
7. Mark the unit complete only when no failed item remains.

A wrong item returns to the pending mastery queue. Correct items are removed. The next phase opens only when the queue is empty, so the final pass requirement is always 100%.

## Zero-unknown-word contract

For each lesson, the allowed English vocabulary is exactly:

- every word completed in earlier lessons;
- every new word introduced by the current lesson;
- ordinary controlled inflections of learned nouns and verbs; and
- story character names explicitly declared from the fixed proper-name list.

Inflections do not introduce a new lexical entry. Learning `plan` as a verb can permit `plans`, `planned`, and `planning`, but never an unrelated form such as `planet`. Declared character names are valid only inside that lesson's reading and question prompts and never count as learned vocabulary.

The validator checks all learner-facing English in:

- word phrases;
- word examples;
- sentence exercises;
- reading titles;
- reading sentences; and
- English answer options.

A course pack is rejected if any token falls outside the cumulative whitelist. Each current lesson word must also occur in the paired reading at least twice.

## Audio contract

Every English word, phrase, example, sentence, reading sentence, complete reading, and English answer option is playable. The initial Windows implementation uses the operating system's English speech service and requires no separately installed runtime.

## Extension boundary

The engine is reusable, but the product is not a user-configurable course platform. Future official stages such as Oxford 5000 or technical reading are added as validated course data that follows the same fixed workflow and rules.

## Curated narrative contract

Normal generated units may use the controlled-context fallback, but official story units are static assets in `assets/course-stories/curated.json`.

Every curated story is required to declare a setup, goal, problem, at least two attempts, a turn, an optional reveal, and a resolution. The deterministic validator also checks sentence-count limits by CEFR level, target-word coverage, exact-form coverage, named-character use, connector variety, repeated sentence openings, duplicate sentences, and the cumulative vocabulary whitelist.

AI can create candidate stories offline through `tools/generate_course_stories.py`. AI is not called by the desktop program or by the release workflow.
