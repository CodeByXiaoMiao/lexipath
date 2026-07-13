# Course rules

LexiPath has one fixed learning path. Users do not edit lesson order, daily volume, pass thresholds, or review rules.

## Mandatory unit sequence

For ordinary Ogden and Oxford lessons:

1. Learn every new word and play its English pronunciation.
2. Pass word-meaning recognition.
3. Pass listening recognition.
4. Pass paired sentence training.
5. Read the reviewed LLM-authored article and play the complete English reading.
6. Pass reading comprehension.
7. Mark the unit complete only when no failed item remains.

A wrong item returns to the pending mastery queue. Correct items are removed. The next phase opens only when the queue is empty, so the final pass requirement is always 100%.

## Foundation exception

The first 15 lessons belong to the `foundation-words` stage. They are controlled introductory sentence drills rather than long reading lessons because the cumulative learned vocabulary is too small to support a coherent 10-or-more-sentence article without introducing unknown words.

These 15 lessons:

- may use deterministic phrase, example, and controlled-sentence templates;
- must be labeled as controlled sentence practice rather than reading articles;
- are excluded from the LLM article bank;
- are excluded from the `--require-llm-readings` coverage gate; and
- must not be counted as missing LLM articles.

The Ogden stage-final assessment is a separately curated long reading and is also outside the ordinary LLM article bank.

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

## LLM reading contract

Vocabulary phrases and example sentences may be produced by deterministic templates, but an ordinary Ogden or Oxford reading article must come from the reviewed static LLM article bank in `assets/course-stories/curated.json`. Template sentences must not be presented as an article.

Every article declares a setup, goal, problem, at least two attempts, a turn, an optional reveal, and a resolution. The deterministic validator checks sentence-count limits by CEFR level, target-word coverage, exact-form coverage, named-character use, connector variety, repeated openings, duplicate sentences, the cumulative vocabulary whitelist, and one Simplified Chinese translation per English sentence.

AI creates candidates offline through `tools/generate_course_stories.py`. The desktop program and normal release workflow do not call an AI service. Strict release finalization uses `--require-llm-readings` and fails when any required ordinary lesson is missing an article.

The migration baseline is 500 required ordinary articles: 133 Ogden, 82 A1, 96 A2, 97 B1, and 92 B2. One A1 article existed when the migration started, leaving 499 to generate. Operational generation and review procedures are defined in `docs/LLM_READING_MAINTENANCE.md`.