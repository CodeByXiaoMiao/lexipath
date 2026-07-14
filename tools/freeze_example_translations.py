#!/usr/bin/env python3
"""Write the reviewed example-translation whole-bank freeze manifest."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from validate_example_translations import (
    FREEZE_MANIFEST_NAME,
    canonical_freeze_payload,
    course_words,
    effective_correction_count,
    fnv1a64,
    load_bank,
    load_json,
)

SCOPE = (
    "course word text, meaning, phrase, primary example, "
    "and effective Chinese translation"
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("course", type=Path)
    parser.add_argument(
        "--bank",
        type=Path,
        default=Path("assets/example-translations"),
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="manifest path; defaults to the translation-bank directory",
    )
    args = parser.parse_args()

    try:
        course = load_json(args.course)
        records = load_bank(args.bank)
        correction_count = effective_correction_count(args.bank)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 1

    by_id = {record["word_id"]: record for record in records}
    expected_ids: set[str] = set()
    issues: list[str] = []
    for _stage, lesson, word in course_words(course):
        word_id = word["id"]
        expected_ids.add(word_id)
        record = by_id.get(word_id)
        if record is None:
            issues.append(f"lesson {lesson['id']} / word {word_id}: translation is missing")
            continue
        expected_hash = fnv1a64(word.get("example", ""))
        if record["english_hash"] != expected_hash:
            issues.append(
                f"lesson {lesson['id']} / word {word_id}: fingerprint mismatch"
            )

    extra = sorted(set(by_id) - expected_ids)
    if extra:
        issues.append(
            f"translation bank contains {len(extra)} records outside the course"
        )
    if issues:
        print("\n".join(issues[:100]), file=sys.stderr)
        return 1

    payload, stage_counts = canonical_freeze_payload(course, by_id)
    manifest = {
        "schema": 1,
        "status": "frozen",
        "record_count": sum(stage_counts.values()),
        "effective_correction_count": correction_count,
        "stage_counts": stage_counts,
        "canonical_sha256": hashlib.sha256(payload).hexdigest(),
        "scope": SCOPE,
    }
    output = args.output or args.bank / FREEZE_MANIFEST_NAME
    output.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(
        f"Wrote {output}: {manifest['record_count']} frozen records, "
        f"{correction_count} effective corrections, "
        f"sha256={manifest['canonical_sha256']}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
