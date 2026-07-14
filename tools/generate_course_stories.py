#!/usr/bin/env python3
"""Generate static LLM-authored LexiPath reading articles and Chinese translations."""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from pathlib import Path
from typing import Any
from urllib.request import Request, urlopen

NAMES = {
    "Alex",
    "Anna",
    "Ben",
    "Emma",
    "Jack",
    "Leo",
    "Lily",
    "Lucy",
    "Mia",
    "Nina",
    "Sam",
    "Tom",
}
CONNECTORS = {
    "after",
    "although",
    "because",
    "before",
    "but",
    "however",
    "if",
    "so",
    "then",
    "though",
    "when",
    "while",
}
IRREGULAR_FORMS = {
    "be": {"am", "is", "are", "was", "were", "been", "being"},
    "begin": {"began", "begun", "beginning", "begins"},
    "buy": {"bought", "buying", "buys"},
    "do": {"did", "done", "doing", "does"},
    "feel": {"felt", "feeling", "feels"},
    "find": {"found", "finding", "finds"},
    "give": {"gave", "given", "giving", "gives"},
    "go": {"went", "gone", "going", "goes"},
    "have": {"had", "having", "has"},
    "hear": {"heard", "hearing", "hears"},
    "leave": {"left", "leaving", "leaves"},
    "make": {"made", "making", "makes"},
    "read": {"reading", "reads"},
    "say": {"said", "saying", "says"},
    "see": {"saw", "seen", "seeing", "sees"},
    "sit": {"sat", "sitting", "sits"},
    "take": {"took", "taken", "taking", "takes"},
    "tell": {"told", "telling", "tells"},
    "think": {"thought", "thinking", "thinks"},
    "write": {"wrote", "written", "writing", "writes"},
}
TOKEN_RE = re.compile(r"[A-Za-z]+(?:'[A-Za-z]+)?")
CJK_RE = re.compile(r"[\u4e00-\u9fff]")


def tokens(text: str) -> list[str]:
    output: list[str] = []
    for raw in TOKEN_RE.findall(text):
        value = raw.lower()
        output.append(value[:-2] if value.endswith("'s") else value)
    return output


def forms(word: str) -> set[str]:
    value = word.lower()
    output = {value} | IRREGULAR_FORMS.get(value, set())
    if " " in value:
        return output
    if value.endswith("e"):
        output |= {value + "s", value[:-1] + "ing", value + "d"}
    elif value.endswith("y") and len(value) > 1 and value[-2] not in "aeiou":
        output |= {value[:-1] + "ies", value[:-1] + "ied", value + "ing"}
    elif (
        len(value) > 2
        and value[-1] not in "aeiouywx"
        and value[-2] in "aeiou"
        and value[-3] not in "aeiou"
    ):
        output |= {value + "s", value + value[-1] + "ed", value + value[-1] + "ing"}
    else:
        output |= {value + "s", value + "ed", value + "ing"}
    return output


def load_json(path: Path, default: Any) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def iter_lessons(course: dict[str, Any]):
    learned: list[str] = []
    for stage in course["stages"]:
        for lesson in stage["lessons"]:
            current = [word["text"] for word in lesson["new_words"]]
            learned.extend(current)
            yield stage, lesson, list(learned)


def level_of(stage: dict[str, Any]) -> str:
    text = (stage.get("id", "") + " " + stage.get("title", "")).upper()
    return next((level for level in ("A1", "A2", "B1", "B2") if level in text), "A1")


def profile(level: str) -> tuple[int, int, int, int]:
    return {
        "A1": (10, 16, 16, 4),
        "A2": (12, 18, 20, 5),
        "B1": (14, 22, 24, 6),
        "B2": (16, 26, 28, 7),
    }[level]


def prompt_vocabulary(known: list[str], targets: list[str], level: str) -> list[str]:
    limits = {
        "A1": (220, 80),
        "A2": (350, 120),
        "B1": (550, 180),
        "B2": (850, 240),
    }
    first_count, recent_count = limits[level]
    candidates = known[:first_count] + known[-recent_count:] + targets
    output: list[str] = []
    seen: set[str] = set()
    for entry in candidates:
        key = entry.lower()
        if key not in seen:
            output.append(entry)
            seen.add(key)
    return output


def allowed_words(known: list[str], characters: list[str]) -> set[str]:
    output = {name.lower() for name in characters}
    for entry in known:
        for part in tokens(entry):
            output |= forms(part)
    return output


def sequence_count(sequence: list[str], target: list[str]) -> int:
    if not target or len(sequence) < len(target):
        return 0
    return sum(
        sequence[index : index + len(target)] == target
        for index in range(len(sequence) - len(target) + 1)
    )


def target_coverage(sentences: list[str], target: str) -> tuple[int, int]:
    target_tokens = tokens(target)
    exact_hits = 0
    exact_sentences = 0
    for sentence in sentences:
        sentence_tokens = tokens(sentence)
        hits = sequence_count(sentence_tokens, target_tokens)
        exact_hits += hits
        exact_sentences += int(hits > 0)
    return exact_hits, exact_sentences


def validate_story(
    story: dict[str, Any], lesson: dict[str, Any], known: list[str]
) -> list[str]:
    issues: list[str] = []
    field = str(story.get("lesson_id", "?"))
    level = story.get("level", "")
    characters = story.get("characters", [])
    sentences = story.get("sentences", [])
    translations = story.get("translations", [])
    arc = story.get("arc", {})

    if story.get("lesson_id") != lesson["id"]:
        issues.append(f"{field}: lesson id mismatch")
    title = story.get("title")
    if not isinstance(title, str) or not title.strip() or not CJK_RE.search(title):
        issues.append(f"{field}: title must be non-empty Simplified Chinese")
    if level not in {"A1", "A2", "B1", "B2"}:
        issues.append(f"{field}: invalid level")
    if (
        not isinstance(characters, list)
        or not 1 <= len(characters) <= 4
        or any(name not in NAMES for name in characters)
        or len(set(characters)) != len(characters)
    ):
        issues.append(f"{field}: invalid characters")
    if not isinstance(sentences, list) or any(not isinstance(item, str) for item in sentences):
        issues.append(f"{field}: sentences must be a string array")
        sentences = []
    if not isinstance(translations, list) or any(
        not isinstance(item, str) for item in translations
    ):
        issues.append(f"{field}: translations must be a string array")
        translations = []

    if level in {"A1", "A2", "B1", "B2"}:
        minimum, maximum, max_words, min_connectors = profile(level)
        if not minimum <= len(sentences) <= maximum:
            issues.append(f"{field}: expected {minimum}-{maximum} sentences")
        if any(len(tokens(sentence)) > max_words for sentence in sentences):
            issues.append(f"{field}: sentence is too long")
        connector_count = len(CONNECTORS & set(tokens(" ".join(sentences))))
        available_connector_count = len(CONNECTORS & allowed_words(known, []))
        required_connectors = min(min_connectors, available_connector_count)
        if connector_count < required_connectors:
            issues.append(
                f"{field}: too few connectors; expected {required_connectors}, found {connector_count}"
            )

    normalized = {sentence.strip().lower() for sentence in sentences}
    if len(normalized) != len(sentences):
        issues.append(f"{field}: duplicate sentence")
    if len(translations) != len(sentences):
        issues.append(f"{field}: expected one Chinese translation per sentence")
    elif any(not item.strip() or not CJK_RE.search(item) for item in translations):
        issues.append(f"{field}: every translation must contain Chinese text")

    openings = [tokens(sentence)[:2] for sentence in sentences]
    if any(
        openings[index] == openings[index + 1] == openings[index + 2]
        for index in range(max(0, len(openings) - 2))
    ):
        issues.append(f"{field}: repeated opening frame")

    required = [
        "setup_sentence",
        "goal_sentence",
        "problem_sentence",
        "attempt_sentences",
        "turn_sentence",
        "resolution_sentence",
    ]
    if not isinstance(arc, dict) or any(key not in arc for key in required):
        issues.append(f"{field}: incomplete narrative arc")
    else:
        indexes = [
            arc["setup_sentence"],
            arc["goal_sentence"],
            arc["problem_sentence"],
            arc["turn_sentence"],
            arc["resolution_sentence"],
        ]
        attempts = arc["attempt_sentences"]
        extra = list(attempts) if isinstance(attempts, list) else []
        if arc.get("reveal_sentence") is not None:
            extra.append(arc["reveal_sentence"])
        if any(
            not isinstance(index, int) or index < 0 or index >= len(sentences)
            for index in indexes + extra
        ):
            issues.append(f"{field}: arc index out of range")
        elif (
            not indexes[0] <= indexes[1] < indexes[2] < indexes[3] < indexes[4]
            or len(extra if arc.get("reveal_sentence") is None else extra[:-1]) < 2
            or any(not indexes[2] < index < indexes[3] for index in attempts)
        ):
            issues.append(f"{field}: invalid narrative order")

    allowed = allowed_words(known, characters if isinstance(characters, list) else [])
    unknown = sorted(set(tokens(" ".join(sentences))) - allowed)
    if unknown:
        issues.append(f"{field}: unknown words: {', '.join(unknown)}")

    all_story_tokens = tokens(" ".join(sentences))
    for name in characters if isinstance(characters, list) else []:
        if all_story_tokens.count(name.lower()) < 2:
            issues.append(f"{field}: character {name} appears fewer than twice")

    for word in lesson["new_words"]:
        exact_hits, exact_sentences = target_coverage(sentences, word["text"])
        if exact_hits < 2 or exact_sentences < 2:
            issues.append(
                f"{field}: target {word['text']} must appear in exact form in at least two sentences"
            )
    return issues


def build_prompt(
    stage: dict[str, Any], lesson: dict[str, Any], known: list[str]
) -> str:
    level = level_of(stage)
    minimum, maximum, max_words, min_connectors = profile(level)
    targets = [
        {"word": word["text"], "meaning": word["meaning"]}
        for word in lesson["new_words"]
    ]
    usable_words = prompt_vocabulary(
        known, [word["text"] for word in lesson["new_words"]], level
    )
    contract = {
        "lesson_id": lesson["id"],
        "level": level,
        "usable_known_words": usable_words,
        "targets": targets,
        "allowed_names": sorted(NAMES),
        "sentence_count": [minimum, maximum],
        "max_words_per_sentence": max_words,
        "minimum_distinct_connectors": min_connectors,
    }
    return (
        "Write one coherent English micro-story for an English learner. Return JSON only. "
        "Use only usable_known_words, ordinary noun/verb inflections, and declared "
        "allowed_names. Every target must appear in exact dictionary form in at least two "
        "different sentences. The story must have a setup, goal, concrete problem, at least "
        "two attempts, a turn, optional reveal, and a resolution that recalls an earlier "
        "object, action, or piece of advice. Vary openings; do not create a list of example "
        "sentences. Give the story a short Simplified Chinese title. Also provide one natural "
        "Simplified Chinese translation for every English sentence. The translations array "
        "must have exactly the same length and order as sentences. JSON schema: "
        "{lesson_id,title,level,characters,sentences,translations,arc:{setup_sentence,"
        "goal_sentence,problem_sentence,attempt_sentences,turn_sentence,reveal_sentence,"
        "resolution_sentence}}. Sentence indexes are zero-based.\nCONTRACT:\n"
        + json.dumps(contract, ensure_ascii=False, separators=(",", ":"))
    )


def parse_model_json(content: str) -> dict[str, Any]:
    text = content.strip()
    if text.startswith("```"):
        text = re.sub(r"^```(?:json)?\s*", "", text)
        text = re.sub(r"\s*```$", "", text)
    value = json.loads(text)
    if not isinstance(value, dict):
        raise ValueError("model response is not a JSON object")
    return value


def call_model(endpoint: str, model: str, token: str, prompt: str) -> dict[str, Any]:
    body = json.dumps(
        {
            "model": model,
            "temperature": 0.7,
            "response_format": {"type": "json_object"},
            "messages": [
                {
                    "role": "system",
                    "content": "You write controlled-vocabulary narrative course content.",
                },
                {"role": "user", "content": prompt},
            ],
        }
    ).encode()
    request = Request(
        endpoint,
        data=body,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        },
    )
    with urlopen(request, timeout=180) as response:
        data = json.load(response)
    return parse_model_json(data["choices"][0]["message"]["content"])


def select_lessons(course: dict[str, Any], args: argparse.Namespace):
    selected = []
    for stage, lesson, known in iter_lessons(course):
        if stage.get("id") == "foundation-words":
            continue
        if lesson["id"].startswith("stage-final-"):
            continue
        if args.lesson and lesson["id"] != args.lesson:
            continue
        if args.stage and stage["id"] != args.stage:
            continue
        selected.append((stage, lesson, known))
    if not (args.lesson or args.stage or args.all or args.validate_only):
        raise SystemExit("choose --lesson, --stage, --all, or --validate-only")
    return selected


def write_bank(path: Path, stories: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(stories, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("course", type=Path)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("assets/course-stories/curated.json"),
    )
    parser.add_argument("--lesson")
    parser.add_argument("--stage")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--require-complete", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--endpoint", default="https://models.github.ai/inference/chat/completions"
    )
    parser.add_argument("--model", default="openai/gpt-4.1")
    parser.add_argument("--token-env", default="GITHUB_TOKEN")
    parser.add_argument("--max-retries", type=int, default=3)
    parser.add_argument("--pause-seconds", type=float, default=1.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    course = load_json(args.course, None)
    bank = load_json(args.output, [])
    by_id = {story["lesson_id"]: story for story in bank}
    selected = select_lessons(course, args)
    all_lessons = {item[1]["id"]: item for item in iter_lessons(course)}

    if args.validate_only:
        issues: list[str] = []
        for story in bank:
            item = all_lessons.get(story.get("lesson_id"))
            if item is None:
                issues.append(f"{story.get('lesson_id')}: lesson not found")
            else:
                issues.extend(validate_story(story, item[1], item[2]))
        if args.require_complete:
            expected = {lesson["id"] for _, lesson, _ in selected}
            missing = sorted(expected - set(by_id))
            if missing:
                issues.append(
                    f"story bank is incomplete: {len(missing)} lessons missing; "
                    f"first items: {', '.join(missing[:20])}"
                )
        if issues:
            print("\n".join(issues), file=sys.stderr)
            return 1
        print(f"Validated {len(bank)} LLM reading articles.")
        return 0

    if args.dry_run:
        for stage, lesson, known in selected:
            print(build_prompt(stage, lesson, known) + "\n")
        return 0

    token = os.getenv(args.token_env)
    if not token:
        raise SystemExit(f"environment variable {args.token_env} is required")

    for stage, lesson, known in selected:
        if args.resume and lesson["id"] in by_id:
            continue
        base_prompt = build_prompt(stage, lesson, known)
        last_issues: list[str] = []
        for _attempt in range(1, args.max_retries + 1):
            retry_context = (
                "\nPrevious validation errors: " + " | ".join(last_issues)
                if last_issues
                else ""
            )
            story = call_model(args.endpoint, args.model, token, base_prompt + retry_context)
            last_issues = validate_story(story, lesson, known)
            if not last_issues:
                break
        else:
            raise SystemExit(f"{lesson['id']}: " + " | ".join(last_issues))
        by_id[lesson["id"]] = story
        write_bank(args.output, [by_id[key] for key in sorted(by_id)])
        print(f"accepted {lesson['id']}")
        time.sleep(args.pause_seconds)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
