#!/usr/bin/env python3
"""
Validates the implementation inventory against the first-stage gate requirements.

First-stage gate: ALL mandatory inventory items must be Verified.
This script replaces the unreliable "grep -c" + "greater than 15" heuristic
used in the stabilize-rc workflow.

Usage: ./ci/validate-inventory.py
Exit: 0 if all mandatory items are Verified, 1 otherwise.
"""

import re
import sys
from pathlib import Path

INVENTORY_PATH = Path("IMPLEMENTATION_INVENTORY.md")
MANDATORY_PREFIXES = ["COMP-"]  # Items required for the agreed release


def parse_inventory(path: Path) -> dict:
    """Parse the implementation inventory and return status counts by ID."""
    inventory = {}
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Find the Verified/Partial/Accepted/Blocked items table
    # Skip header lines and separator lines
    in_table = False
    for line in lines:
        line = line.strip()
        # Detect table start (header row)
        if re.match(r"^\| ID \| Item", line):
            in_table = True
            continue
        if in_table and line.startswith("|----"):
            continue
        if in_table and line.startswith("| "):
            # Parse: | ID | Item | Crate/Module | Current Status | Blockers |
            parts = [p.strip() for p in line.split("|")]
            if len(parts) >= 5:
                item_id = parts[1]  # Second field is ID
                status = parts[4]   # Fifth field is Current Status
                if item_id.startswith("COMP-") or item_id.startswith("ADR-"):
                    inventory[item_id] = status
    return inventory


def validate_inventory(inventory: dict) -> tuple:
    """
    Validate that all mandatory inventory items are Verified.

    Returns (passed: bool, report: str)
    """
    # Parse the inventory
    items = parse_inventory(INVENTORY_PATH)

    # Count items by status
    verified_count = 0
    accepted_count = 0
    partial_count = 0
    blocked_count = 0
    other_count = 0
    missing_mandatory = []

    for item_id, status in items.items():
        # Determine if this is a mandatory item
        is_mandatory = any(item_id.startswith(prefix) for prefix in MANDATORY_PREFIXES)

        if not is_mandatory:
            other_count += 1
            continue

        if status == "Verified":
            verified_count += 1
        elif status == "Accepted":
            accepted_count += 1
        elif status == "Partial":
            partial_count += 1
        elif status == "Blocked":
            blocked_count += 1
        else:
            other_count += 1

        # Check mandatory items
        if is_mandatory and status != "Verified":
            missing_mandatory.append((item_id, status))

    total_mandatory = verified_count + accepted_count + partial_count + blocked_count + len(
        [i for i in items.values() if any(i.startswith(prefix) for prefix in MANDATORY_PREFIXES)]
    )

    # GATE CRITERIA: All mandatory items must be Verified
    # The first-stage gate requires ALL mandatory items to be Verified.
    # No heuristics (e.g., "greater than 15") are acceptable.
    all_verified = len(missing_mandatory) == 0

    # Generate report
    report_lines = [
        "=" * 60,
        "IMPLEMENTATION INVENTORY VALIDATION REPORT",
        "=" * 60,
        f"Total items parsed from inventory: {len(items)}",
        f"Mandatory items Verified: {verified_count}",
        f"Mandatory items Accepted: {accepted_count}",
        f"Mandatory items Partial: {partial_count}",
        f"Mandatory items Blocked: {blocked_count}",
        f"Mandatory items other: {other_count}",
        "",
    ]

    if all_verified:
        report_lines.extend [
            "RESULT: PASS — All mandatory inventory items are Verified.",
            "The first-stage completion gate requirement is satisfied.",
        ]
    else:
        report_lines.extend(
            [
                "RESULT: FAIL — Not all mandatory items are Verified.",
                f"Missing Verified status for {len(missing_mandatory)} mandatory item(s):",
            ]
        )
        for item_id, status in missing_mandatory:
            report_lines.append(f"  - {item_id}: {status}")

        # Also list all mandatory items and their statuses for completeness
        report_lines.append("")
        report_lines.append("All mandatory item statuses:")
        for item_id, status in sorted(items.items()):
            prefix_match = any(item_id.startswith(p) for p in MANDATORY_PREFIXES)
            marker = " <-- MANDATORY" if prefix_match else ""
            report_lines.append(f"  {item_id}: {status}{marker}")

    report_lines.append("")
    report_lines.append("=" * 60)

    # Output the report to stdout
    report = "\n".join(report_lines)
    print(report)

    return all_verified, report


def main():
    inventory = parse_inventory(INVENTORY_PATH)
    passed, report = validate_inventory(inventory)

    # Also output JSON-compatible result for workflow consumption
    # Parse the items again for the JSON output
    items = parse_inventory(INVENTORY_PATH)
    verified_count = sum(
        1 for s in items.values() if s == "Verified"
        and any(s.startswith(prefix) for prefix in MANDATORY_PREFIXES)
    )

    # Count mandatory items total
    mandatory_ids = [
        item_id for item_id in items.keys()
        if any(item_id.startswith(prefix) for prefix in MANDATORY_PREFIXES)
    ]
    total_mandatory = len(mandatory_ids)

    result = {
        "all_verified": passed,
        "verified_count": verified_count,
        "total_mandatory": total_mandatory,
        "missing_count": total_mandatory - verified_count,
        "mandatory_ids": mandatory_ids,
    }
    print(f"\n__RESULT__:{str(result)}")

    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()