# LLM reading maintenance runbook

This document is the operational checklist for generating, reviewing, validating, and releasing LexiPath LLM-authored reading articles.

## Scope and exclusions

The course contains 516 lessons in total.

- `foundation-words`: 15 introductory lessons. These remain controlled sentence drills and do not require long LLM articles.
- `stage-final-ogden-850`: one separately curated long assessment reading.
- All other Ogden and Oxford lessons: 500 ordinary lessons that require reviewed LLM-authored articles.

At the start of the migration, one Oxford A1 article was already accepted, leaving 499 articles to generate:

| Batch | Required articles | Initially remaining |
|---|---:|---:|
| `ogden-850` | 133 | 133 |
| `oxford-a1` | 82 | 81 |
| `oxford-a2` | 96 | 96 |
| `oxford-b1` | 97 | 97 |
| `oxford-b2` | 92 | 92 |
| **Total** | **500** | **499** |

The remaining values above are migration-baseline values. Do not treat them as a live counter after generation begins.

## Selected model

The current selected authoring model is:

```text
Provider: GitHub Models
Endpoint: https://models.github.ai/inference/chat/completions
Model: openai/gpt-4.1
```

A model change is a maintenance decision, not an incidental command-line change. Record the previous model, replacement model, reason, and a reviewed comparison sample in the pull request before changing the committed authoring policy.

## Required generation order

Generate and review the bank in this fixed order:

```text
1. Ogden 850
2. Oxford A1
3. Oxford A2
4. Oxford B1
5. Oxford B2
```

Ogden is part of the 499-article migration and must not be skipped.

## Preparation

Generate the current finalized course first so lesson IDs, meanings, and target order are stable.

Set the authoring token only in the local shell:

```powershell
$env:GITHUB_TOKEN = "..."
```

Do not commit tokens, generated request logs containing tokens, or private endpoint credentials.

Before generating a stage, validate the existing bank:

```powershell
python tools/generate_course_stories.py course.json `
  --validate-only
```

## Stage commands

Each command uses `--resume`, so an interrupted run skips already accepted lessons and continues from the static article bank.

### Ogden 850

```powershell
python tools/generate_course_stories.py course.json `
  --stage ogden-850 `
  --model openai/gpt-4.1 `
  --resume `
  --output assets/course-stories/curated.json
```

### Oxford A1

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-a1 `
  --model openai/gpt-4.1 `
  --resume `
  --output assets/course-stories/curated.json
```

### Oxford A2

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-a2 `
  --model openai/gpt-4.1 `
  --resume `
  --output assets/course-stories/curated.json
```

### Oxford B1

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-b1 `
  --model openai/gpt-4.1 `
  --resume `
  --output assets/course-stories/curated.json
```

### Oxford B2

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-b2 `
  --model openai/gpt-4.1 `
  --resume `
  --output assets/course-stories/curated.json
```

## Review and commit policy

Do not wait until all 499 articles are generated before reviewing them. Treat each stage as a migration batch and use smaller review commits inside a stage when the JSON diff becomes difficult to inspect.

For every batch:

1. Run deterministic validation.
2. Review every rejected lesson before retrying; never bypass the validator by weakening the contract merely to increase acceptance rate.
3. Review all titles and Chinese translations for obvious semantic mismatch.
4. Manually read a representative sample from the beginning, middle, and end of the batch.
5. Check that stories are not repeating the same plot structure, names, objects, or endings excessively.
6. Confirm every article remains appropriate for its CEFR stage.
7. Commit the accepted static JSON with the model name and stage in the commit or pull-request notes.
8. Record accepted and remaining counts in the pull request.

After each stage or review batch:

```powershell
python tools/generate_course_stories.py course.json `
  --validate-only
```

The validator must pass before the article bank is committed.

## Completion gate

When all required ordinary lessons have articles, run the complete-bank check:

```powershell
python tools/generate_course_stories.py course.json `
  --validate-only `
  --require-complete
```

Then perform strict course finalization:

```powershell
cargo run --release -- `
  --finalize-catalog `
  --input raw-course.json `
  --output course.json `
  --require-llm-readings
```

A release advertised as the complete LLM-reading edition must not be produced without both commands succeeding.

## Release invariants

- The desktop application never calls an LLM.
- Normal application startup never generates or rewrites an article.
- Release CI does not silently author missing content.
- All accepted English articles and Chinese translations are committed as static data.
- The first 15 foundation lessons remain clearly labeled controlled sentence drills, not reading articles.
- Template-generated phrases and examples may support vocabulary practice, but must never be displayed or documented as a reading article.
- The Ogden stage-final long reading remains separately curated and is not counted as one of the 500 ordinary LLM articles.
