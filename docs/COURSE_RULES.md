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

- every word completed in earlier lessons; and
- every new word introduced by the current lesson.

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
