# LLM reading authoring

LexiPath treats a reading article as authored course content, not as a set of template examples.

The desktop application never calls an LLM. A model is used only in an offline authoring step. This step can be driven by GitHub Models in CI or by the current GPT model directly in the local workspace; the local path does not require `GITHUB_TOKEN`. Accepted articles and their Chinese translations are committed as static JSON, then Rust validates them again during course finalization.

## Content flow

1. Generate and finalize a `course.json` so every lesson has stable target words and meanings.
2. Run `tools/generate_course_stories.py` for one lesson, one stage, or the complete course.
3. The model writes one continuous English story plus one Simplified Chinese translation per sentence. The story must stay in one setting, carry the same characters and concrete objects through the event, and use causal or temporal links between actions.
4. The Python validator rejects weak narrative structure, missing targets, repeated sentence frames, translation-count mismatches, and empty/non-Chinese translations. Rust additionally checks connector density, a persistent scene/entity chain, and adjacent-sentence links.
5. Accepted articles are saved to `assets/course-stories/curated.json`.
6. Rust finalization applies the static articles and validates them again.
7. A release intended to contain only LLM articles must use `--require-llm-readings`; finalization fails if any required ordinary lesson is missing an article.

AI is not a runtime or release-time dependency. A model outage can delay content authoring, but cannot silently change an existing release.

## Coverage policy

The first 15 lessons in the `foundation-words` stage are controlled introductory sentence drills. Their cumulative vocabulary is intentionally too small for a coherent 10-or-more-sentence article, so they are excluded from LLM-article coverage and from `--require-llm-readings`.

The Ogden stage-final assessment is a separately curated long reading and is also excluded from the ordinary LLM article bank.

All other Ogden and Oxford lessons require reviewed LLM-authored articles. The migration baseline is:

| Stage | Ordinary lessons requiring LLM articles | Accepted at migration start | Remaining at migration start |
|---|---:|---:|---:|
| Ogden 850 | 133 | 0 | 133 |
| Oxford A1 | 82 | 1 | 81 |
| Oxford A2 | 96 | 0 | 96 |
| Oxford B1 | 97 | 0 | 97 |
| Oxford B2 | 92 | 0 | 92 |
| **Total** | **500** | **1** | **499** |

The table is a migration baseline, not a live counter. After every accepted batch, validate the current bank and record the new coverage in the pull request or batch commit.

The current repository snapshot contains 6 accepted articles (5 Ogden and 1 Oxford A1), so 494 ordinary lessons still require authored articles. Strict release validation intentionally fails until those assets are present.

Generation order is fixed:

```text
Ogden 850 -> Oxford A1 -> Oxford A2 -> Oxford B1 -> Oxford B2
```

Ogden must not be omitted: the 499-article migration total includes its 133 ordinary lessons.

The current selected authoring model is `openai/gpt-4.1` through GitHub Models. Changing the model requires an intentional maintenance decision and must be recorded in the pull request together with the old model, new model, reason, and a reviewed comparison sample.

Detailed batch commands and acceptance rules are maintained in `docs/LLM_READING_MAINTENANCE.md`.

## Narrative contract

Each article must contain a setup, character goal, concrete problem, at least two attempts, a turn, an optional reveal, and a resolution that recalls an earlier object, action, or piece of advice. Every current target must appear in at least two separate sentences. Sentence length and connector requirements increase by CEFR level.

An article is not accepted merely because every target word appears twice. Independent example sentences are rejected: after removing the target words, most sentences must still share a character, object, place, or causal event with nearby sentences. The resolution must follow from the attempts and turn, rather than introducing a new topic.

The `translations` array must match `sentences` one-for-one and remain in the same order.
Every English sentence must have terminal punctuation. Every Chinese translation must use Chinese sentence punctuation, contain no untranslated English fragment, and pass the rejected-artifact and unnatural-frame checks in both the authoring script and Rust finalization.

## Commands

Validate the committed bank without a network request:

```powershell
python tools/generate_course_stories.py course.json --validate-only
```

Require complete article coverage for all required ordinary lessons:

```powershell
python tools/generate_course_stories.py course.json --validate-only --require-complete
```

Show the prompt for one lesson:

```powershell
python tools/generate_course_stories.py course.json --lesson a1-unit-047 --dry-run
```

Generate or replace one lesson with the selected model:

```powershell
$env:GITHUB_TOKEN = "..."
python tools/generate_course_stories.py course.json `
  --lesson a1-unit-047 `
  --model openai/gpt-4.1 `
  --output assets/course-stories/curated.json
```

Generate the remaining articles in a stage and resume safely after interruption:

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-a1 `
  --model openai/gpt-4.1 `
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
