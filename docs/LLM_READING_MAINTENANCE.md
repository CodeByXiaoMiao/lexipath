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

The remaining values above are migration-baseline values. Do not treat them as a live counter after generation begins. Record live accepted and remaining counts in the pull request after every batch.

## Authoring methods and model record

The current migration uses direct ChatGPT authoring for reviewed batches:

```text
Provider: OpenAI ChatGPT
Model: GPT-5.6 Thinking
Method: direct assistant-authored static JSON
External authoring token: not required
```

The OpenAI-compatible generator remains available as an optional local batch tool. Its default GitHub Models settings require a local `GITHUB_TOKEN`, but that token is not required when an assistant directly authors the JSON and submits it through the repository connector.

Every article batch must record the actual provider, model, method, accepted count, and validation result in the commit or pull-request notes. A model change is a maintenance decision, not an incidental command-line change. Record the previous model, replacement model, reason, and a reviewed comparison sample before changing the authoring policy.

## Early-vocabulary connector rule

A story must never introduce an unlearned connector merely to satisfy a stylistic quota. The CEFR profile defines a maximum connector target, while the effective requirement is capped by the number of approved narrative connectors already available in the cumulative learned vocabulary.

As the learner acquires `but`, `if`, `so`, `when`, and later connectors, the required variety rises automatically. This rule applies identically in the Python authoring validator and the Rust release validator.

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

Before authoring a batch, validate the existing bank:

```powershell
python tools/generate_course_stories.py course.json `
  --validate-only
```

Direct assistant authoring does not require an external token. When the optional GitHub Models generator is used, set its authoring token only in the local shell:

```powershell
$env:GITHUB_TOKEN = "..."
```

Do not commit tokens, generated request logs containing tokens, or private endpoint credentials.

## Optional API stage commands

These commands are retained for maintainers who deliberately choose the API generator. Each command uses `--resume`, so an interrupted run skips already accepted lessons and continues from the static article bank.

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

## Direct assistant batch procedure

For direct assistant authoring:

1. Read the finalized lesson targets and cumulative vocabulary from `course.json`.
2. Author a coherent English article and one Simplified Chinese translation per sentence.
3. Add the article to `assets/course-stories/curated.json` without altering previously accepted records.
4. Run deterministic validation locally.
5. Submit a small reviewable batch to the draft pull request.
6. Wait for Rust tests, full-course finalization, and Windows workflows to pass before starting the next batch.

A small batch normally contains five to twenty articles. Use smaller batches for early constrained-vocabulary lessons or whenever the JSON diff becomes difficult to review.

## Review and commit policy

Do not wait until all articles are generated before reviewing them. Treat each stage as a migration batch and use smaller review commits inside a stage.

For every batch:

1. Run deterministic validation.
2. Review every rejected lesson before retrying; never bypass the validator merely to increase acceptance rate.
3. Review every title and Chinese translation for semantic mismatch.
4. Manually read all articles in small batches; for larger batches, read representative samples from the beginning, middle, and end in addition to automated checks.
5. Check that stories are not repeating the same plot structure, names, objects, or endings excessively.
6. Confirm every article remains appropriate for its CEFR stage.
7. Commit the accepted static JSON with provider, model, method, and stage in the commit or pull-request notes.
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
