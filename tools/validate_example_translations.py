#!/usr/bin/env python3
"""Validate LexiPath's reviewed Chinese example-sentence translation bank."""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

CJK_RE = re.compile(r"[\u4e00-\u9fff]")
REJECTED_ARTIFACTS = (
    "这是一个书。",
    "这是一个食物。",
    "这是一 yard。",
    "这是一十亿。",
    "这是几把剪刀。",
    "他是女性。",
    "这是一个水平。",
    "这是一个膝盖。",
    "例句中",
    "机器翻译",
    "（例句中文译文缺失）",
)
CORRECTION_FILE = "review-corrections.tsv"


def fnv1a64(value: str) -> str:
    result = 0xCBF29CE484222325
    for byte in value.encode("utf-8"):
        result ^= byte
        result = (result * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{result:016x}"


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_file(path: Path, expected_method: str) -> list[dict[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if len(lines) < 2 or lines[0] != "# schema=1":
        raise ValueError(f"{path}: unsupported or missing schema")
    if lines[1] != f"authoring_method\t{expected_method}":
        raise ValueError(f"{path}: unsupported or missing authoring method")

    records: list[dict[str, str]] = []
    for line_number, line in enumerate(lines[2:], start=3):
        if not line.strip():
            continue
        fields = line.split("\t", maxsplit=2)
        if len(fields) != 3 or any(not value.strip() for value in fields):
            raise ValueError(f"{path}:{line_number}: invalid TSV record")
        if not re.fullmatch(r"[0-9a-f]{16}", fields[1]):
            raise ValueError(f"{path}:{line_number}: invalid English fingerprint")
        records.append(
            {"word_id": fields[0], "english_hash": fields[1], "chinese": fields[2]}
        )
    return records


def load_bank(directory: Path) -> list[dict[str, str]]:
    files = sorted(
        path for path in directory.glob("*.tsv") if path.name != CORRECTION_FILE
    )
    if not files:
        raise ValueError(f"no translation bank TSV files found in {directory}")

    records: list[dict[str, str]] = []
    base_ids: set[str] = set()
    for path in files:
        for record in load_file(path, "direct-llm-reviewed"):
            word_id = record["word_id"]
            if word_id in base_ids:
                raise ValueError(f"{path}: duplicate base word_id {word_id!r}")
            base_ids.add(word_id)
            records.append(record)

    by_id = {record["word_id"]: record for record in records}
    correction_path = directory / CORRECTION_FILE
    if correction_path.exists():
        correction_ids: set[str] = set()
        for correction in load_file(correction_path, "direct-llm-correction"):
            word_id = correction["word_id"]
            if word_id in correction_ids:
                raise ValueError(
                    f"{correction_path}: duplicate correction word_id {word_id!r}"
                )
            correction_ids.add(word_id)
            if word_id not in by_id:
                raise ValueError(
                    f"{correction_path}: correction has no base record for {word_id!r}"
                )
            by_id[word_id] = correction

    return [by_id[record["word_id"]] for record in records]


def course_words(course: dict[str, Any]):
    for stage in course["stages"]:
        for lesson in stage["lessons"]:
            for word in lesson["new_words"]:
                yield stage, lesson, word


def validate_chinese(field: str, value: Any) -> list[str]:
    issues: list[str] = []
    if not isinstance(value, str) or not value.strip():
        return [f"{field}: Chinese translation is empty"]
    text = value.strip()
    if not CJK_RE.search(text):
        issues.append(f"{field}: Chinese translation contains no Chinese text")
    if not text.endswith(("。", "！", "？")):
        issues.append(f"{field}: Chinese translation needs Chinese punctuation")
    for artifact in REJECTED_ARTIFACTS:
        if artifact in text:
            issues.append(f"{field}: rejected artifact {artifact!r}")
    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("course", type=Path)
    parser.add_argument(
        "--bank",
        type=Path,
        default=Path("assets/example-translations"),
    )
    args = parser.parse_args()

    course = load_json(args.course)
    try:
        records = load_bank(args.bank)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    issues: list[str] = []

    by_id: dict[str, dict[str, str]] = {}
    for index, record in enumerate(records):
        field = f"records[{index}]"
        word_id = record["word_id"].strip()
        if word_id in by_id:
            issues.append(f"{field}: duplicate effective word_id {word_id!r}")
            continue
        by_id[word_id] = record
        issues.extend(validate_chinese(field, record["chinese"]))

    expected_ids: set[str] = set()
    for _stage, lesson, word in course_words(course):
        word_id = word["id"]
        expected_ids.add(word_id)
        field = f"lesson {lesson['id']} / word {word_id}"
        record = by_id.get(word_id)
        if record is None:
            issues.append(f"{field}: reviewed translation is missing")
            continue
        expected_hash = fnv1a64(word.get("example", ""))
        if record["english_hash"] != expected_hash:
            issues.append(
                f"{field}: English fingerprint mismatch: "
                f"bank={record['english_hash']}, expected={expected_hash}, "
                f"example={word.get('example', '')!r}"
            )
        issues.extend(validate_chinese(field, record["chinese"]))

    extra = sorted(set(by_id) - expected_ids)
    if extra:
        issues.append(
            f"translation bank has {len(extra)} records not present in the course; "
            f"first items: {', '.join(extra[:20])}"
        )

    if issues:
        print("\n".join(issues[:400]), file=sys.stderr)
        if len(issues) > 400:
            print(f"... {len(issues) - 400} more issues", file=sys.stderr)
        return 1

    correction_count = len(load_file(args.bank / CORRECTION_FILE, "direct-llm-correction"))
    print(
        f"Validated {len(expected_ids)} reviewed example translations, including "
        f"{correction_count} reviewed corrections; coverage and English matching are complete."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
