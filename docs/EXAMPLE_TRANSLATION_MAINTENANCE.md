# Reviewed example-translation maintenance

LexiPath example sentences use reviewed static Simplified Chinese translations. The desktop application must not create example translations by joining dictionary glosses at runtime.

## Source of truth

The immutable stage banks are stored under:

```text
assets/example-translations/*.tsv
```

Each base record is keyed by the stable course word ID and contains the stable word ID, a deterministic fingerprint of the exact English example, and the reviewed Chinese sentence:

```text
w-a\tb15c9a9fd56f0185\t这是一本书。
```

The English fingerprint is intentional. If an example is rewritten, the old Chinese translation is rejected instead of being silently reused for a different sentence. The FNV-1a fingerprint is only a deterministic change detector; it is not a security mechanism.

## Reviewed correction layer

Naturalness issues found during later manual review are recorded in:

```text
assets/example-translations/review-corrections.tsv
```

A correction must:

- target an existing base word ID;
- include the fingerprint of the exact current English example;
- contain a complete reviewed Chinese sentence; and
- replace the effective base record during validation and at runtime.

The validator rejects duplicate corrections and corrections without a matching base record. Keeping corrections separate makes later review changes small and inspectable without rewriting the full 3,083-record baseline.

## Authoring policy

- Chinese must be a natural sentence, not a word-for-word concatenation of dictionary meanings.
- Preserve the English subject, negation, tense, modality, quantity, and sentence type.
- Use natural Chinese measure words and punctuation.
- Translate the sentence actually shown to the learner. Dictionary metadata may be noisy and must not be copied mechanically.
- Ambiguous examples require a deliberate reviewed sense; do not let a runtime fallback choose one.
- Do not include part-of-speech labels, English placeholder words, translation notes, or model commentary.
- One effective record is required for every course word, including the foundation lessons.

The current full-course baseline contains 3,083 word records. Treat this as a release snapshot, not a permanently hard-coded course size. The validator derives the expected count from the finalized `course.json`.

## Catalog lifecycle

Course generation has three separate responsibilities:

```text
import raw catalog once
→ finalize and rewrite course content once
→ validate the packaged course without rewriting it
```

Neither the release workflow nor normal application startup may run the mutating finalization step twice. Re-finalizing an already finalized course can change an example after its translation has been reviewed and is therefore treated as a build defect.

## Human-review export

Do not review the bank by opening the large TSV files in isolation. Export the effective translation after applying the correction layer together with its course context:

```powershell
python tools/export_example_translation_review.py course.json `
  --stage foundation-words `
  --stage ogden-850 `
  --output foundation-ogden-review.tsv
```

The export contains the stage, lesson ID, stable word ID, target word, exact English example, effective Chinese translation, and whether the correction layer overrides the base record. Use `--format csv` when spreadsheet software handles CSV more conveniently. A stage ID may be supplied more than once to create a focused review batch.

The export is review material only. Editing it does not change the application. Accepted fixes must still be recorded in `review-corrections.tsv`, and any rewritten English example must update its fingerprint.

## Validation

After building the finalized course, run:

```powershell
python tools/validate_example_translations.py course.json
```

The command fails for:

- a missing or duplicate effective word ID;
- an English example that no longer exactly matches the course;
- an empty or non-Chinese translation;
- missing Chinese sentence punctuation;
- a stale record that is no longer present in the course;
- an invalid correction target; or
- known runtime-machine-translation artifacts.

Rust course finalization performs the same coverage and exact-English checks. Example-translation coverage is mandatory for every release; there is no optional flag and no machine-translation fallback.

## Change procedure

1. Import and finalize a deterministic `course.json` from pinned course sources.
2. Identify records whose English example changed or whose Chinese was reported as unnatural.
3. Add a small reviewed record to `review-corrections.tsv`; update the base stage bank only when rebuilding the full reviewed baseline.
4. Run the Python validator.
5. Run Rust formatting, tests, full-course finalization, startup validation, and Windows packaging.
6. Record the reviewed range and any intentionally chosen ambiguous senses in the pull request.

LLM reading-article generation remains paused whenever example-translation coverage or quality is incomplete.
