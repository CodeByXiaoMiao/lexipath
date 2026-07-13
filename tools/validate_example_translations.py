#!/usr/bin/env python3
"""Validate LexiPath's reviewed Chinese example-sentence translation bank."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from collections import Counter
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
    "这项任务难得令人难以置信。",
    "这是一套系列丛书。",
    "这是内部部分。",
    "成功的前景很好。",
    "这是一个电子设备。",
    "这辆车有一个轮胎瘪了。",
    "这是一处严重的伤势。",
    "这是一份奖品。",
    "这是一篇不错的译文。",
    "这是一个数量。",
    "这是一个背景。",
    "他受了很痛的伤。",
    "那是一种轻慢。",
    "这是一个可以接受的结果。",
    "它们大体相同。",
    "它们肯定完全相同。",
    "例句中",
    "机器翻译",
    "（例句中文译文缺失）",
)
CORRECTION_PREFIX = "review-corrections"
FREEZE_MANIFEST_NAME = "freeze-manifest.json"


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


def correction_paths(directory: Path) -> list[Path]:
    return sorted(
        directory.glob(f"{CORRECTION_PREFIX}*.tsv"),
        key=lambda path: (path.name != "review-corrections.tsv", path.name),
    )


def load_bank(directory: Path) -> list[dict[str, str]]:
    files = sorted(
        path
        for path in directory.glob("*.tsv")
        if not path.name.startswith(CORRECTION_PREFIX)
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
    for correction_path in correction_paths(directory):
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


def effective_correction_count(directory: Path) -> int:
    return len(
        {
            record["word_id"]
            for path in correction_paths(directory)
            for record in load_file(path, "direct-llm-correction")
        }
    )


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


def canonical_freeze_payload(
    course: dict[str, Any],
    by_id: dict[str, dict[str, str]],
) -> tuple[bytes, dict[str, int]]:
    lines: list[str] = []
    stage_counts: Counter[str] = Counter()
    for stage, lesson, word in course_words(course):
        stage_id = stage["id"]
        stage_counts[stage_id] += 1
        record = by_id[word["id"]]
        frozen = {
            "stage_id": stage_id,
            "lesson_id": lesson["id"],
            "word_id": word["id"],
            "text": word.get("text", ""),
            "meaning": word.get("meaning", ""),
            "phrase": word.get("phrase", ""),
            "example": word.get("example", ""),
            "chinese": record["chinese"],
        }
        lines.append(
            json.dumps(
                frozen,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
        )
    return ("\n".join(lines) + "\n").encode("utf-8"), dict(stage_counts)


def validate_freeze_manifest(
    manifest_path: Path,
    course: dict[str, Any],
    by_id: dict[str, dict[str, str]],
    correction_count: int,
) -> list[str]:
    if not manifest_path.is_file():
        return [f"{manifest_path}: whole-bank freeze manifest is missing"]

    try:
        manifest = load_json(manifest_path)
    except (OSError, json.JSONDecodeError) as error:
        return [f"{manifest_path}: invalid freeze manifest: {error}"]

    issues: list[str] = []
    payload, stage_counts = canonical_freeze_payload(course, by_id)
    digest = hashlib.sha256(payload).hexdigest()
    expected = {
        "schema": 1,
        "status": "frozen",
        "record_count": sum(stage_counts.values()),
        "effective_correction_count": correction_count,
        "stage_counts": stage_counts,
        "canonical_sha256": digest,
    }
    for key, value in expected.items():
        if manifest.get(key) != value:
            issues.append(
                f"{manifest_path}: {key} mismatch; "
                f"manifest={manifest.get(key)!r}, expected={value!r}"
            )
    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("course", type=Path)
    parser.add_argument(
        "--bank",
        type=Path,
        default=Path("assets/example-translations"),
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help=(
            "whole-bank freeze manifest; defaults to "
            "assets/example-translations/freeze-manifest.json"
        ),
    )
    args = parser.parse_args()

    course = load_json(args.course)
    try:
        records = load_bank(args.bank)
        correction_count = effective_correction_count(args.bank)
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
    english_to_chinese: dict[str, tuple[str, str]] = {}
    for _stage, lesson, word in course_words(course):
        word_id = word["id"]
        expected_ids.add(word_id)
        field = f"lesson {lesson['id']} / word {word_id}"
        record = by_id.get(word_id)
        if record is None:
            issues.append(f"{field}: reviewed translation is missing")
            continue

        english = word.get("example", "")
        expected_hash = fnv1a64(english)
        if record["english_hash"] != expected_hash:
            issues.append(
                f"{field}: English fingerprint mismatch: "
                f"bank={record['english_hash']}, expected={expected_hash}, "
                f"example={english!r}"
            )
        issues.extend(validate_chinese(field, record["chinese"]))

        if english.rstrip().endswith("?") and not record["chinese"].rstrip().endswith("？"):
            issues.append(f"{field}: English question must remain a Chinese question")

        previous = english_to_chinese.get(english)
        if previous is None:
            english_to_chinese[english] = (word_id, record["chinese"])
        elif previous[1] != record["chinese"]:
            issues.append(
                f"{field}: identical English example has inconsistent Chinese; "
                f"{previous[0]}={previous[1]!r}, {word_id}={record['chinese']!r}"
            )

    extra = sorted(set(by_id) - expected_ids)
    if extra:
        issues.append(
            f"translation bank has {len(extra)} records not present in the course; "
            f"first items: {', '.join(extra[:20])}"
        )

    manifest_path = args.manifest or args.bank / FREEZE_MANIFEST_NAME
    if not issues:
        issues.extend(
            validate_freeze_manifest(
                manifest_path,
                course,
                by_id,
                correction_count,
            )
        )

    if issues:
        print("\n".join(issues[:400]), file=sys.stderr)
        if len(issues) > 400:
            print(f"... {len(issues) - 400} more issues", file=sys.stderr)
        return 1

    print(
        f"Validated frozen whole bank: {len(expected_ids)} reviewed example "
        f"translations, {correction_count} effective corrections, exact English "
        f"fingerprints, cross-entry consistency, and canonical manifest digest."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
