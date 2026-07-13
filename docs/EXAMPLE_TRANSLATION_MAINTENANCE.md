# Reviewed example-translation maintenance

LexiPath example sentences use reviewed static Simplified Chinese translations. The desktop application must not create example translations by joining dictionary glosses at runtime.

## Source of truth

The reviewed bank is:

```text
assets/example-translations/*.tsv
```

Each record is keyed by the stable course word ID and contains the stable word ID, a deterministic fingerprint of the exact English example, and the reviewed Chinese sentence:

```text
w-a\tb15c9a9fd56f0185\t这是一本书。
```

The English fingerprint is intentional. If an example is rewritten, the old Chinese translation is rejected instead of being silently reused for a different sentence. The FNV-1a fingerprint is only a deterministic change detector; it is not a security mechanism.

## Authoring policy

- Chinese must be a natural sentence, not a word-for-word concatenation of dictionary meanings.
- Preserve the English subject, negation, tense, modality, quantity, and sentence type.
- Use natural Chinese measure words and punctuation.
- Translate the sentence actually shown to the learner. Dictionary metadata may be noisy and must not be copied mechanically.
- Ambiguous examples require a deliberate reviewed sense; do not let a runtime fallback choose one.
- Do not include part-of-speech labels, English placeholder words, translation notes, or model commentary.
- One record is required for every course word, including the foundation lessons.

The current full-course baseline contains 3,083 word records. Treat this as a release snapshot, not a permanently hard-coded course size. The validator derives the expected count from the finalized `course.json`.

## Validation

After building the finalized course, run:

```powershell
python tools/validate_example_translations.py course.json
```

The command fails for:

- a missing or duplicate word ID;
- an English example that no longer exactly matches the course;
- an empty or non-Chinese translation;
- missing Chinese sentence punctuation;
- a stale record that is no longer present in the course; or
- known runtime-machine-translation artifacts.

Rust course finalization performs the same coverage and exact-English checks. Example-translation coverage is mandatory for every release; there is no optional flag and no machine-translation fallback.

## Change procedure

1. Finalize a deterministic `course.json` from pinned course sources.
2. Identify records whose English example changed or whose Chinese was reported as unnatural.
3. Rewrite those translations as reviewed static Chinese.
4. Run the Python validator.
5. Run Rust formatting, tests, full-course finalization, and Windows packaging.
6. Record the reviewed range and any intentionally chosen ambiguous senses in the pull request.

LLM reading-article generation remains paused whenever example-translation coverage or quality is incomplete.
