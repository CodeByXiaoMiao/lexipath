#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import traceback
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

ENDPOINT = "https://models.github.ai/inference/chat/completions"
API_VERSION = "2026-03-10"
DEFAULT_MODEL = "openai/gpt-4.1-mini"
BATCH_SIZE = 5

SYSTEM_PROMPT = """You are a strict English curriculum reviewer for Chinese beginner and intermediate learners.
The material is controlled English: short and repetitive sentences are intentionally allowed.
Do not criticize a sentence merely for being simple or repetitive.

Review every supplied lesson for:
1. grammatical correctness;
2. natural and semantically plausible use of each target word or phrase;
3. agreement between the Chinese core meaning and the English usage;
4. correct part of speech, articles, countability, verb objects, and word order;
5. misleading or nonsensical examples;
6. malformed text or duplicated answer choices.

Use severity "error" when material is ungrammatical, uses a target with the wrong meaning or part of speech, or would teach a learner something misleading.
Use severity "warning" for awkward but still correct wording, excessive metalinguistic fallback, or weak teaching value.
Return JSON only, with this exact shape:
{"issues":[{"lesson_id":"...","severity":"error|warning","word":"...","message":"..."}]}
Return an empty issues array when all lessons in the batch are acceptable.
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("course", type=Path)
    parser.add_argument("--report-json", type=Path, default=Path("ai-course-review.json"))
    parser.add_argument("--report-md", type=Path, default=Path("ai-course-review.md"))
    parser.add_argument("--model", default=os.environ.get("GITHUB_MODELS_MODEL", DEFAULT_MODEL))
    return parser.parse_args()


def compact_lessons(course: dict[str, Any]) -> list[dict[str, Any]]:
    lessons: list[dict[str, Any]] = []
    for stage in course.get("stages", []):
        for lesson in stage.get("lessons", []):
            lessons.append(
                {
                    "stage": stage.get("id"),
                    "lesson_id": lesson.get("id"),
                    "title": lesson.get("title"),
                    "words": [
                        {
                            "word": item.get("text"),
                            "meaning": item.get("meaning"),
                            "phrase": item.get("phrase"),
                            "example": item.get("example"),
                        }
                        for item in lesson.get("new_words", [])
                    ],
                    "sentence_exercises": [
                        item.get("text") for item in lesson.get("sentences", [])
                    ],
                    "reading": lesson.get("reading", {}).get("sentences", []),
                    "answer_options": [
                        question.get("options", [])
                        for question in lesson.get("reading", {}).get("questions", [])
                    ],
                }
            )
    return lessons


def chunks(items: list[dict[str, Any]], size: int):
    for index in range(0, len(items), size):
        yield index // size + 1, items[index : index + size]


def request_review(token: str, model: str, batch: list[dict[str, Any]]) -> dict[str, Any]:
    payload = {
        "model": model,
        "temperature": 0,
        "max_tokens": 3500,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {
                "role": "user",
                "content": "Review this batch:\n"
                + json.dumps(batch, ensure_ascii=False, separators=(",", ":")),
            },
        ],
    }
    request = urllib.request.Request(
        ENDPOINT,
        data=json.dumps(payload, ensure_ascii=False).encode("utf-8"),
        method="POST",
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": API_VERSION,
            "Content-Type": "application/json",
            "User-Agent": "LexiPath-course-review",
        },
    )

    for attempt in range(5):
        try:
            with urllib.request.urlopen(request, timeout=180) as response:
                body = json.loads(response.read().decode("utf-8"))
            content = body["choices"][0]["message"]["content"].strip()
            if content.startswith("```"):
                content = content.split("\n", 1)[1].rsplit("```", 1)[0].strip()
            result = json.loads(content)
            if not isinstance(result.get("issues"), list):
                raise ValueError("model response has no issues array")
            return result
        except urllib.error.HTTPError as error:
            if error.code not in {408, 429, 500, 502, 503, 504} or attempt == 4:
                details = error.read().decode("utf-8", errors="replace")
                raise RuntimeError(f"GitHub Models HTTP {error.code}: {details}") from error
        except (urllib.error.URLError, TimeoutError) as error:
            if attempt == 4:
                raise RuntimeError(f"GitHub Models request failed: {error}") from error
        time.sleep(2 ** attempt)

    raise RuntimeError("GitHub Models request failed after retries")


def validate_issues(
    raw_issues: list[dict[str, Any]], valid_lesson_ids: set[str]
) -> list[dict[str, str]]:
    output: list[dict[str, str]] = []
    for raw in raw_issues:
        lesson_id = str(raw.get("lesson_id", "")).strip()
        severity = str(raw.get("severity", "warning")).strip().lower()
        if lesson_id not in valid_lesson_ids:
            continue
        if severity not in {"error", "warning"}:
            severity = "warning"
        output.append(
            {
                "lesson_id": lesson_id,
                "severity": severity,
                "word": str(raw.get("word", "")).strip(),
                "message": str(raw.get("message", "")).strip(),
            }
        )
    return output


def write_reports(
    report_json: Path,
    report_md: Path,
    model: str,
    lesson_count: int,
    issues: list[dict[str, str]],
    *,
    completed_batches: int,
    total_batches: int,
    review_error: str | None = None,
) -> None:
    errors = [item for item in issues if item["severity"] == "error"]
    warnings = [item for item in issues if item["severity"] == "warning"]
    payload = {
        "model": model,
        "lessons_reviewed": lesson_count,
        "completed_batches": completed_batches,
        "total_batches": total_batches,
        "complete": completed_batches == total_batches and review_error is None,
        "review_error": review_error,
        "error_count": len(errors),
        "warning_count": len(warnings),
        "issues": issues,
    }
    report_json.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8"
    )

    lines = [
        "# LexiPath AI Course Review",
        "",
        f"- Model: `{model}`",
        f"- Lessons in course: {lesson_count}",
        f"- Batches completed: {completed_batches}/{total_batches}",
        f"- Complete: {payload['complete']}",
        f"- Errors: {len(errors)}",
        f"- Warnings: {len(warnings)}",
        "",
    ]
    if review_error:
        lines.extend(["## Review process error", "", "```", review_error, "```", ""])
    if issues:
        lines.append("## Issues")
        lines.append("")
    for item in issues:
        label = "ERROR" if item["severity"] == "error" else "WARNING"
        word = f" — `{item['word']}`" if item["word"] else ""
        lines.append(
            f"- **{label}** `{item['lesson_id']}`{word}: {item['message']}"
        )
    report_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        print("GITHUB_TOKEN is required", file=sys.stderr)
        return 2

    course = json.loads(args.course.read_text(encoding="utf-8"))
    lessons = compact_lessons(course)
    valid_ids = {str(lesson["lesson_id"]) for lesson in lessons}
    all_issues: list[dict[str, str]] = []
    total_batches = (len(lessons) + BATCH_SIZE - 1) // BATCH_SIZE
    completed_batches = 0

    try:
        write_reports(
            args.report_json,
            args.report_md,
            args.model,
            len(lessons),
            all_issues,
            completed_batches=0,
            total_batches=total_batches,
        )
        for batch_number, batch in chunks(lessons, BATCH_SIZE):
            print(f"AI review batch {batch_number}/{total_batches}", flush=True)
            response = request_review(token, args.model, batch)
            all_issues.extend(validate_issues(response["issues"], valid_ids))
            completed_batches = batch_number
            write_reports(
                args.report_json,
                args.report_md,
                args.model,
                len(lessons),
                all_issues,
                completed_batches=completed_batches,
                total_batches=total_batches,
            )
            time.sleep(1)
    except Exception as error:  # noqa: BLE001 - CI must preserve the partial report.
        review_error = "".join(
            traceback.format_exception_only(type(error), error)
        ).strip()
        print(review_error, file=sys.stderr)
        write_reports(
            args.report_json,
            args.report_md,
            args.model,
            len(lessons),
            all_issues,
            completed_batches=completed_batches,
            total_batches=total_batches,
            review_error=review_error,
        )
        return 1

    errors = [item for item in all_issues if item["severity"] == "error"]
    warnings = [item for item in all_issues if item["severity"] == "warning"]
    print(
        f"AI reviewed {len(lessons)} lessons: {len(errors)} errors, "
        f"{len(warnings)} warnings"
    )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
