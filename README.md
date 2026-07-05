# LexiPath

LexiPath is a focused Windows English-learning application written in Rust.

## Fixed learning path

1. IPA foundation
2. Ogden Basic English 850
3. Oxford 3000
4. Later official stages added through the same validated course format

The application does not let users change lesson order, daily volume, pass thresholds, or the mandatory learning workflow.

## Non-negotiable rules

- Every new vocabulary unit has paired controlled reading.
- Learner-facing English is checked against the cumulative learned-word whitelist.
- Reading titles, sentences, examples, exercises, and English options cannot introduce an unlearned word.
- Every current lesson word must appear in the paired reading at least twice.
- Wrong answers stay in the mastery queue until answered correctly.
- A phase only passes when the remaining error queue reaches zero: final mastery is 100%.
- Words, phrases, examples, sentences, and complete readings can be played aloud.

## Current MVP

The `bootstrap/mvp` branch contains:

- an eframe/egui Windows desktop UI;
- a data-driven course-pack model;
- a strict zero-unknown-word validator with tests;
- a fixed learning state machine;
- Windows English speech playback using the operating system voice;
- bundled SQLite progress storage;
- one minimal, fully controlled example lesson; and
- Windows CI for formatting, tests, and release compilation.

The example lesson intentionally uses only six words. It proves the learning engine and content validator before large-scale course authoring begins.

## Build

Install the stable Rust toolchain, then run:

```powershell
cargo run --release
```

The release executable stores progress in a `data` directory beside `LexiPath.exe`.

## Course contract

See [`docs/COURSE_RULES.md`](docs/COURSE_RULES.md).
