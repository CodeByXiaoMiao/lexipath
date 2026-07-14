#!/usr/bin/env python3
"""Export effective example translations with course context for human review."""
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path
from typing import Any, TextIO

from example_translation_review_bank import fnv1a64, load_effective_bank

FIELDS = (
    "stage",
    "lesson_id",
    "word_id",
    "target_word",
    "english_example",
    "current_chinese_translation",
    "overridden_by_correction",
)


def iter_course_words(course: dict[str, Any]):
    for stage in course["stages"]:
        for lesson in stage["lessons"]:
            for word in lesson["new_words"]:
                yield stage, lesson, word


def open_output(path: Path | None) -> tuple[TextIO, bool]:
    if path is None:
        return sys.stdout, False
    path.parent.mkdir(parents=True, exist_ok=True)
    return path.open("w", encoding="utf-8", newline=""), True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("course", type=Path)
    parser.add_argument(
        "--bank",
        type=Path,
        default=Path("assets/example-translations"),
    )
    parser.add_argument(
        "--stage",
        action="append",
        dest="stages",
        help="Export only this stage ID. Repeat to include multiple stages.",
    )
    parser.add_argument("--output", type=Path, help="Write to this file instead of stdout.")
    parser.add_argument(
        "--format",
        choices=("tsv", "csv"),
        default="tsv",
        help="Output format (default: tsv).",
    )
    args = parser.parse_args()

    try:
        course = json.loads(args.course.read_text(encoding="utf-8"))
        effective, overridden_ids = load_effective_bank(args.bank)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 1

    selected_stages = set(args.stages or [])
    available_stages = {stage["id"] for stage in course["stages"]}
    unknown_stages = sorted(selected_stages - available_stages)
    if unknown_stages:
        print("unknown stage IDs: " + ", ".join(unknown_stages), file=sys.stderr)
        return 1

    output, should_close = open_output(args.output)
    delimiter = "\t" if args.format == "tsv" else ","
    writer = csv.DictWriter(
        output,
        fieldnames=FIELDS,
        delimiter=delimiter,
        lineterminator="\n",
    )
    writer.writeheader()

    exported = 0
    try:
        for stage, lesson, word in iter_course_words(course):
            if selected_stages and stage["id"] not in selected_stages:
                continue
            word_id = word["id"]
            record = effective.get(word_id)
            if record is None:
                raise ValueError(f"missing translation for {word_id!r}")
            expected_hash = fnv1a64(word.get("example", ""))
            if record["english_hash"] != expected_hash:
                raise ValueError(
                    f"stale translation for {word_id!r}: "
                    f"bank={record['english_hash']}, expected={expected_hash}"
                )
            writer.writerow(
                {
                    "stage": stage["id"],
                    "lesson_id": lesson["id"],
                    "word_id": word_id,
                    "target_word": word["text"],
                    "english_example": word.get("example", ""),
                    "current_chinese_translation": record["chinese"],
                    "overridden_by_correction": (
                        "yes" if word_id in overridden_ids else "no"
                    ),
                }
            )
            exported += 1
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    finally:
        if should_close:
            output.close()

    print(f"Exported {exported} example translations.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
