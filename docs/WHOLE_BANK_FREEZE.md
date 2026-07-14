# Whole-bank example-translation freeze

After every vocabulary stage has completed systematic review, the combined 3,083-entry
word/example baseline is locked as one unit.

## Frozen scope

The manifest is stored at:

```text
assets/example-translations/freeze-manifest.json
```

Its SHA-256 digest covers the course-ordered values below:

- stage ID and lesson ID;
- stable word ID and display text;
- normalized meaning and phrase;
- primary English example; and
- effective reviewed Simplified Chinese translation.

Reading titles, article sentences, questions, and article metadata are intentionally not
part of this digest. New reviewed reading articles can therefore be added without silently
changing the frozen vocabulary baseline.

## Final cross-stage cleanup layer

High-confidence issues found only when all stages are reviewed together are stored in:

```text
assets/example-templates/final-freeze.tsv
assets/example-translations/review-corrections-zz-final-freeze.tsv
```

The final template layer has priority over stage-specific templates. It must contain only
reviewed corrections keyed by stable `word_id`; it is not an unreviewed catch-all.

## Regenerating the manifest

Regeneration is an explicit human-review action:

```powershell
python tools/freeze_example_translations.py course.json
python tools/validate_example_translations.py course.json
```

The generator is never run automatically by CI. Otherwise an accidental vocabulary change
could replace its own expected digest.

An intentional frozen-baseline change must:

1. update the appropriate reviewed template and correction layer;
2. finalize the complete course;
3. export and review the effective whole bank;
4. regenerate the manifest from that exact finalized course;
5. run Python validation, Rust tests, full-course finalization, and all release builds; and
6. record the new digest and validation checkpoint in the pull request.

The validator also rejects stale English fingerprints, missing/extra translations,
inconsistent Chinese for identical English examples, lost question punctuation, and known
mechanical translation artifacts.
