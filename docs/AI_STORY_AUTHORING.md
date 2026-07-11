# LLM reading authoring

LexiPath treats a reading article as authored course content, not as a set of template examples.

The desktop application never calls an LLM. An OpenAI-compatible model is used only in an offline authoring step. Accepted articles and their Chinese translations are committed as static JSON, then Rust validates them again during course finalization.

## Content flow

1. Generate and finalize a `course.json` so every lesson has stable target words and meanings.
2. Run `tools/generate_course_stories.py` for one lesson, one stage, or the complete course.
3. The model writes one coherent English story plus one Simplified Chinese translation per sentence.
4. The Python validator rejects unknown words, weak narrative structure, missing targets, repeated sentence frames, translation-count mismatches, and empty/non-Chinese translations.
5. Accepted articles are saved to `assets/course-stories/curated.json`.
6. Rust finalization applies the static articles and validates them again.
7. A release intended to contain only LLM articles must use `--require-llm-readings`; finalization fails if any ordinary lesson is missing an article.

AI is not a runtime or release-time dependency. A model outage can delay content authoring, but cannot silently change an existing release.

## Narrative contract

Each article must contain a setup, character goal, concrete problem, at least two attempts, a turn, an optional reveal, and a resolution that recalls an earlier object, action, or piece of advice. Every current target must appear in at least two separate sentences. Sentence length and connector requirements increase by CEFR level.

The `translations` array must match `sentences` one-for-one and remain in the same order.

## Commands

Validate the committed bank without a network request:

```powershell
python tools/generate_course_stories.py course.json --validate-only
```

Require complete article coverage for all ordinary lessons:

```powershell
python tools/generate_course_stories.py course.json --validate-only --require-complete
```

Show the prompt for one lesson:

```powershell
python tools/generate_course_stories.py course.json --lesson a1-unit-047 --dry-run
```

Generate or replace one lesson with GitHub Models:

```powershell
$env:GITHUB_TOKEN = "..."
python tools/generate_course_stories.py course.json `
  --lesson a1-unit-047 `
  --output assets/course-stories/curated.json
```

Generate the remaining articles in a stage and resume safely after interruption:

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-a1 `
  --resume
```

After the bank is complete, finalize a release in strict LLM-reading mode:

```powershell
cargo run --release -- `
  --finalize-catalog `
  --input raw-course.json `
  --output course.json `
  --require-llm-readings
```

Template-generated phrases and example sentences may remain in vocabulary practice, but they are not classified as reading articles.
