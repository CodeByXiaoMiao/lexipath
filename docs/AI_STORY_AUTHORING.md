# AI story authoring

LexiPath uses AI only as an offline content author. The desktop application and GitHub Actions build do not call an AI service.

## Content flow

1. Finalize a `course.json` so every lesson has a stable word order and one reviewed target meaning.
2. Run `tools/generate_course_stories.py` for selected lessons.
3. The tool sends the cumulative learned vocabulary, current target entries, level profile, and narrative requirements to an OpenAI-compatible chat-completions endpoint.
4. Every response is rejected unless deterministic validation passes.
5. Accepted stories are stored in `assets/course-stories/curated.json` and reviewed as normal Git changes.
6. Rust finalization applies those static stories and validates them again before packaging.

AI is never a release-time dependency. An unavailable model can delay authoring, but it cannot change or break an existing build.

## Narrative contract

A curated story must contain:

- one setup and character goal;
- one concrete problem;
- at least two attempts;
- a turn that changes the situation;
- an optional later reveal;
- a resolution that refers back to an earlier object, action, or piece of advice;
- varied sentence openings and the required number of narrative connectors;
- every target entry in exact dictionary form in at least two different sentences.

Declared proper names come from a small fixed list. They are allowed only inside the reading and its question prompts, do not count as learned vocabulary, and cannot be used to bypass the word whitelist.

Ordinary grammatical inflections of learned nouns and verbs are allowed. For example, learning `plan` as a verb opens `plans`, `planned`, and `planning`. It does not open unrelated words such as `planet`.

## Examples

Validate the committed story bank without making a network request:

```powershell
python tools/generate_course_stories.py course.json --validate-only
```

Show the exact prompt for one lesson:

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

Generate the remaining stories in a stage while preserving completed work:

```powershell
python tools/generate_course_stories.py course.json `
  --stage oxford-a1 `
  --resume
```

The tool writes after every accepted lesson, so an interrupted batch can continue with `--resume`.
