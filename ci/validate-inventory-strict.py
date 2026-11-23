#!/usr/bin/env python3
"""Fail-closed validator for Omni's fixed 26-item first-stage inventory.

This checks the inventory's exact requirement IDs, status/detail tables, summary
counts, and selected source-backed claims. It does not grant formal approval,
production Authenticode trust, or release qualification.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

EXPECTED_GATE_IDS = {
    *(f"COMP-{number:03d}" for number in range(1, 17)),
    *(f"COMP-{number:03d}" for number in range(19, 25)),
    "INST-002",
    "INST-003",
    "PLAT-001",
    "ADR-0002",
}
EXCLUDED_IDS = {"COMP-017", "COMP-018"}
EXPECTED_TOTAL = 26
VALID_STATUSES = {"Verified", "Partial", "Accepted", "Blocked"}


def _table_cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def _section(content: str, heading_prefix: str) -> list[str]:
    lines = content.splitlines()
    start = next((i for i, line in enumerate(lines) if line.strip().startswith(heading_prefix)), None)
    if start is None:
        return []
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    return lines[start + 1 : end]


def _identifier(value: str) -> bool:
    return bool(re.fullmatch(r"(?:COMP|INST|PLAT)-\d{3}|ADR-\d{4}", value))


def parse_status_records(content: str) -> tuple[dict[str, str], list[str]]:
    lines = _section(content, "## Requirement Identifier Status Summary")
    records: dict[str, str] = {}
    errors: list[str] = []
    if not lines:
        return records, ["Missing 'Requirement Identifier Status Summary' section."]

    for line in lines:
        if not line.lstrip().startswith("|"):
            continue
        cells = _table_cells(line)
        if len(cells) < 2 or not _identifier(cells[0]):
            continue
        if cells[0] in records:
            errors.append(f"Status table contains duplicate ID {cells[0]}.")
        else:
            records[cells[0]] = cells[1]

    if not records:
        errors.append("Status table contains no requirement records.")
    return records, errors


def parse_summary(content: str) -> tuple[dict[str, int], str, list[str]]:
    lines = _section(content, "## Summary")
    counts: dict[str, int] = {}
    verified_listing = ""
    errors: list[str] = []
    for line in lines:
        if not line.lstrip().startswith("|"):
            continue
        cells = _table_cells(line)
        if len(cells) < 2:
            continue
        label = cells[0].replace("*", "").strip()
        value = cells[1].replace("*", "").strip()
        if label in {"Verified", "Partial", "Accepted", "Blocked", "Total"}:
            try:
                counts[label] = int(value)
            except ValueError:
                errors.append(f"Summary count for {label} is invalid: {value!r}.")
        if label == "Verified" and len(cells) >= 3:
            verified_listing = cells[2]

    for name in ("Verified", "Partial", "Accepted", "Blocked", "Total"):
        if name not in counts:
            errors.append(f"Summary is missing the {name} count.")
    if not verified_listing:
        errors.append("Summary is missing the Verified identifier listing.")
    return counts, verified_listing, errors


def expand_identifier_listing(listing: str) -> tuple[set[str], list[str]]:
    expanded: set[str] = set()
    errors: list[str] = []
    for raw in listing.split(","):
        token = raw.strip()
        range_match = re.fullmatch(r"([A-Z]+)-(\d{3})[–-](\d{3})", token)
        single_match = re.fullmatch(r"([A-Z]+)-(\d{3,4})", token)
        if range_match:
            prefix, first, last = range_match.groups()
            start, stop = int(first), int(last)
            if stop < start:
                errors.append(f"Reversed range in Verified listing: {token!r}.")
                continue
            expanded.update(f"{prefix}-{number:03d}" for number in range(start, stop + 1))
        elif single_match:
            prefix, digits = single_match.groups()
            expanded.add(f"{prefix}-{int(digits):0{len(digits)}d}")
        else:
            errors.append(f"Unparseable Verified listing token: {token!r}.")
    return expanded, errors


def parse_detail_ids(content: str, heading_prefix: str) -> tuple[list[str], list[str]]:
    lines = _section(content, heading_prefix)
    ids: list[str] = []
    if not lines:
        return ids, [f"Missing detail section {heading_prefix!r}."]
    for line in lines:
        if not line.lstrip().startswith("|"):
            continue
        cells = _table_cells(line)
        if cells and _identifier(cells[0]):
            ids.append(cells[0])
    duplicates = sorted(item for item, count in Counter(ids).items() if count > 1)
    errors = [f"{heading_prefix} contains duplicate IDs: {duplicates}."] if duplicates else []
    return ids, errors


def source_evidence_errors(repository_root: Path) -> list[str]:
    errors: list[str] = []
    placeholders = {
        "compiler/omni-pkg/src/main.rs": ("TODO: Implement", "omni-pkg package manager"),
        "compiler/omni-build/src/main.rs": ("TODO: Implement", "omni-build build tool"),
    }
    for relative, markers in placeholders.items():
        path = repository_root / relative
        if not path.is_file():
            errors.append(f"COMP-019 source evidence is missing: {relative}.")
            continue
        source = path.read_text(encoding="utf-8")
        found = [marker for marker in markers if marker in source]
        if found:
            errors.append(
                f"COMP-019 source evidence is not substantiated: {relative} contains "
                f"placeholder marker(s) {found!r}."
            )
    return errors


def validate_inventory(inventory_path: Path, repository_root: Path | None = None) -> dict[str, Any]:
    content = inventory_path.read_text(encoding="utf-8")
    records, errors = parse_status_records(content)
    summary_counts, verified_listing, summary_errors = parse_summary(content)
    errors.extend(summary_errors)

    expected_records = EXPECTED_GATE_IDS | EXCLUDED_IDS
    present_records = set(records)
    missing = sorted(expected_records - present_records)
    unexpected = sorted(present_records - expected_records)
    if missing:
        errors.append(f"Missing requirement status records: {missing}.")
    if unexpected:
        errors.append(f"Unexpected requirement status records: {unexpected}.")

    for item_id in sorted(EXPECTED_GATE_IDS):
        status = records.get(item_id)
        if status not in VALID_STATUSES:
            errors.append(f"{item_id} has unsupported or missing status {status!r}.")

    exclusion_status = {
        "COMP-017": "N/A (duplicate of COMP-011; not a separate mandatory requirement)",
        "COMP-018": "N/A (identifier does not exist)",
    }
    for item_id, expected_status in exclusion_status.items():
        if records.get(item_id) != expected_status:
            errors.append(f"{item_id} must have exclusion status {expected_status!r}.")

    actual_counts = Counter(records.get(item_id) for item_id in EXPECTED_GATE_IDS)
    for label in ("Verified", "Partial", "Accepted", "Blocked"):
        actual = actual_counts.get(label, 0)
        if summary_counts.get(label) != actual:
            errors.append(f"Summary {label} count is {summary_counts.get(label)!r}; records contain {actual}.")
    if summary_counts.get("Total") != EXPECTED_TOTAL:
        errors.append(f"Summary Total must equal {EXPECTED_TOTAL}; got {summary_counts.get('Total')!r}.")

    listed_verified, listing_errors = expand_identifier_listing(verified_listing)
    errors.extend(listing_errors)
    actual_verified = {
        item_id for item_id in EXPECTED_GATE_IDS if records.get(item_id) == "Verified"
    }
    if listed_verified != actual_verified:
        errors.append(
            "Verified summary IDs disagree with status records: "
            f"missing={sorted(actual_verified - listed_verified)}, "
            f"extra={sorted(listed_verified - actual_verified)}."
        )

    detail_sections = (
        ("Verified", "## Verified Items"),
        ("Accepted", "## Accepted Items"),
        ("Blocked", "## Blocked Items"),
        ("N/A", "## Excluded Identifiers"),
    )
    for status, heading in detail_sections:
        detail_ids, detail_errors = parse_detail_ids(content, heading)
        errors.extend(detail_errors)
        expected_ids = {
            item_id for item_id, actual in records.items()
            if (actual == status if status != "N/A" else actual.startswith("N/A"))
        }
        if set(detail_ids) != expected_ids:
            errors.append(
                f"{heading} IDs disagree with status records: "
                f"missing={sorted(expected_ids - set(detail_ids))}, "
                f"extra={sorted(set(detail_ids) - expected_ids)}."
            )

    if repository_root is not None:
        errors.extend(source_evidence_errors(repository_root))

    verified_ids = sorted(actual_verified)
    accepted_ids = sorted(i for i in EXPECTED_GATE_IDS if records.get(i) == "Accepted")
    blocked_ids = sorted(i for i in EXPECTED_GATE_IDS if records.get(i) == "Blocked")
    partial_ids = sorted(i for i in EXPECTED_GATE_IDS if records.get(i) == "Partial")
    all_verified = (
        not errors
        and len(verified_ids) == EXPECTED_TOTAL
        and not accepted_ids
        and not blocked_ids
        and not partial_ids
        and present_records == expected_records
    )
    return {
        "all_verified": all_verified,
        "mandatory_total": EXPECTED_TOTAL,
        "verified_count": len(verified_ids),
        "verified_ids": verified_ids,
        "accepted_ids": accepted_ids,
        "blocked_ids": blocked_ids,
        "partial_ids": partial_ids,
        "excluded_ids": sorted(EXCLUDED_IDS),
        "errors": errors,
    }


def render_report(result: dict[str, Any]) -> str:
    lines = [
        "OMNI STRICT FIRST-STAGE INVENTORY VALIDATION",
        f"Mandatory denominator: {result['mandatory_total']}",
        f"Verified: {result['verified_count']}",
        f"Accepted: {result['accepted_ids']}",
        f"Blocked: {result['blocked_ids']}",
        f"Partial: {result['partial_ids']}",
        f"Excluded (not counted): {result['excluded_ids']}",
        "",
    ]
    if result["all_verified"]:
        lines.extend([
            "RESULT: INVENTORY PASS — all 26 mandatory records are Verified.",
            "This is not formal owner approval or production signing qualification.",
        ])
    else:
        lines.append("RESULT: FAIL — inventory and/or source-backed evidence is incomplete.")
        lines.extend(f"- {error}" for error in result["errors"])
        if not result["errors"]:
            lines.append("- At least one mandatory record is not Verified.")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=Path("IMPLEMENTATION_INVENTORY.md"))
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--json", action="store_true", help="Print the machine-readable result.")
    parser.add_argument(
        "--informational",
        action="store_true",
        help="Always exit 0 after reporting; for non-gating readiness summaries only.",
    )
    args = parser.parse_args()
    result = validate_inventory(args.inventory, args.root)
    print(render_report(result))
    if args.json:
        print(json.dumps(result, sort_keys=True))
    return 0 if result["all_verified"] or args.informational else 1


if __name__ == "__main__":
    sys.exit(main())
