"""Shared loading helpers for reviewed example-translation tools."""
from __future__ import annotations

import re
from pathlib import Path

CORRECTION_PREFIX = "review-corrections"
HASH_RE = re.compile(r"[0-9a-f]{16}")


def fnv1a64(value: str) -> str:
    result = 0xCBF29CE484222325
    for byte in value.encode("utf-8"):
        result ^= byte
        result = (result * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{result:016x}"


def load_tsv(path: Path, expected_method: str) -> dict[str, dict[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if len(lines) < 2 or lines[0] != "# schema=1":
        raise ValueError(f"{path}: unsupported or missing schema")
    if lines[1] != f"authoring_method\t{expected_method}":
        raise ValueError(f"{path}: unsupported or missing authoring method")

    records: dict[str, dict[str, str]] = {}
    for line_number, line in enumerate(lines[2:], start=3):
        if not line.strip():
            continue
        fields = line.split("\t", maxsplit=2)
        if len(fields) != 3 or any(not field.strip() for field in fields):
            raise ValueError(f"{path}:{line_number}: invalid TSV record")
        word_id, english_hash, chinese = fields
        if not HASH_RE.fullmatch(english_hash):
            raise ValueError(f"{path}:{line_number}: invalid English fingerprint")
        if word_id in records:
            raise ValueError(f"{path}:{line_number}: duplicate word_id {word_id!r}")
        records[word_id] = {"english_hash": english_hash, "chinese": chinese}
    return records


def load_effective_bank(directory: Path) -> tuple[dict[str, dict[str, str]], set[str]]:
    base: dict[str, dict[str, str]] = {}
    for path in sorted(directory.glob("*.tsv")):
        if path.name.startswith(CORRECTION_PREFIX):
            continue
        for word_id, record in load_tsv(path, "direct-llm-reviewed").items():
            if word_id in base:
                raise ValueError(f"{path}: duplicate base word_id {word_id!r}")
            base[word_id] = record

    if not base:
        raise ValueError(f"no translation bank TSV files found in {directory}")

    effective = dict(base)
    overridden_ids: set[str] = set()
    correction_paths = sorted(
        directory.glob(f"{CORRECTION_PREFIX}*.tsv"),
        key=lambda path: (path.name != "review-corrections.tsv", path.name),
    )
    for correction_path in correction_paths:
        corrections = load_tsv(correction_path, "direct-llm-correction")
        unknown = sorted(set(corrections) - set(base))
        if unknown:
            raise ValueError(
                f"{correction_path}: corrections have no base record; first items: "
                + ", ".join(unknown[:20])
            )
        effective.update(corrections)
        overridden_ids.update(corrections)
    return effective, overridden_ids
