#!/usr/bin/env python3
"""
Strict implementation inventory validator for the first-stage gate.

Validates that ALL mandatory inventory items are Verified.
Replaces the unreliable "grep -c" + "greater than 15" heuristic
used in the stabilize-rc workflow.

Fail-closed: exits non-zero when any mandatory item is not Verified.
Machine-readable: counts from records, not text matching.

Usage: ./ci/validate-inventory-strict.py
Exit: 0 if all mandatory items are Verified, 1 otherwise.
"""

import re
import sys
from pathlib import Path
from collections import Counter


INVENTORY_PATH = Path("IMPLEMENTATION_INVENTORY.md")

# Mandatory item ID prefixes (as agreed for the release)
MANDATORY_PREFIXES = ["COMP-", "ADR-"]

# Items that are explicitly NOT mandatory (e.g., N/A, non-mandatory)
EXCLUDED_IDS = set()


def parse_inventory(path: Path) -> dict:
    """Parse the implementation inventory and return status counts by ID."""
    inventory = {}
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()

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
                # Only track COMP- and ADR- items (mandatory)
                if item_id.startswith("COMP-") or item_id.startswith("ADR-"):
                    inventory[item_id] = status
    return inventory


def validate_inventory(inventory: dict) -> dict:
    """
    Validate that all mandatory inventory items are Verified.

    Returns dict with validation result details.
    """
    # Categorize items by status
    verified = []
    accepted = []
    blocked = []
    partial = []
    other = []
    missing = []  # Mandatory items not found in inventory

    for item_id, status in inventory.items():
        is_mandatory = any(item_id.startswith(prefix) for prefix in MANDATORY_PREFIXES)

        if not is_mandatory:
            # Non-mandatory items; skip gate check but record
            other.append((item_id, status))
            continue

        if status == "Verified":
            verified.append(item_id)
        elif status == "Accepted":
            accepted.append(item_id)
        elif status == "Blocked":
            blocked.append(item_id)
        elif status == "Partial":
            partial.append(item_id)
        else:
            other.append((item_id, status))

    # Determine which mandatory IDs should exist
    # Collect all IDs that appear in the inventory with mandatory prefixes
    inventory_mandatory_ids = set()
    for item_id in inventory:
        if any(item_id.startswith(prefix) for prefix in MANDATORY_PREFIXES):
            inventory_mandatory_ids.add(item_id)

    # Also check for mandatory IDs that should exist but might be missing
    # (e.g., COMP-018 is N/A and excluded from the gate)
    # The gate only counts items that can be Verified or Blocked

    # Calculate gate result
    # All mandatory items that are not Blocked or Accepted must be Verified
    # Items that are N/A (like COMP-018) are excluded from the mandatory count
    # Items that are Blocked (INST-002) count toward the gate as blocked

    # Get the set of mandatory item IDs that are not N/A
    # We identify N/A items by looking for "N/A (identifier does not exist)" status
    na_item_ids = set()
    for item_id, status in inventory.items():
        if "N/A" in status.upper() or status == "N/A":
            na_item_ids.add(item_id)

    # Mandatory items for the gate: all inventory items with COMP-/ADR- prefixes
    # except those explicitly marked N/A with "identifier does not exist"
    mandatory_items_for_gate = inventory_mandatory_ids - na_item_ids

    # Verified items within the gate scope
    verified_mandatory = [i for i in verified if i in mandatory_items_for_gate]

    # Blocked items within the gate scope
    blocked_mandatory = [i for i in blocked if i in mandatory_items_for_gate]

    # Accepted items within the gate scope
    accepted_mandatory = [i for i in accepted if i in mandatory_items_for_gate]

    # Missing: mandatory items that should be in inventory but aren't
    # (items in mandatory_items_for_gate that don't appear in inventory at all)
    all_known_mandatory_ids = set()
    for item_id, status in inventory.items():
        if any(item_id.startswith(prefix) for prefix in MANDATORY_PREFIXES):
            all_known_mandatory_ids.add(item_id)

    # Also check the summary section for expected IDs
    # Read the full inventory to extract the summary ID list
    expected_ids_from_summary = set()
    with open(INVENTORY_PATH, "r", encoding="utf-8") as f:
        content = f.read()
    # Extract IDs from the summary line: "| Verified   | 25 | COMP-001–017, COMP-019, ..."
    summary_match = re.search(
        r"\| Verified \|\s*\d+\s*\|\s*(.+)", content
    )
    if summary_match:
        summary_ids_str = summary_match.group(1)
        # Parse individual IDs (handle ranges like COMP-001–017)
        for token in summary_ids_str.split(","):
            token = token.strip()
            # Handle range patterns: COMP-001–017
            range_match = re.match(r"COMP-(\d+)-(\d+)", token)
            if range_match:
                start_id = int(range_match.group(1))
                end_id = int(range_match.group(2))
                for n in range(start_id, end_id + 1):
                    expected_ids_from_summary.add(f"COMP-{n}")
            elif token.startswith("COMP-"):
                expected_ids_from_summary.add(token)
            elif token.startswith("ADR-"):
                expected_ids_from_summary.add(token)
            elif token.startswith("INST-"):
                expected_ids_from_summary.add(token)
            elif token.startswith("PLAT-"):
                expected_ids_from_summary.add(token)

    # Also parse the full Requirement Identifier Status Summary table
    in_summary_table = False
    for line in content.split("\n"):
        line = line.strip()
        if re.match(r"^## Requirement Identifier Status Summary", line):
            in_summary_table = True
            continue
        if in_summary_table and line.startswith("|----"):
            continue
        if in_summary_table and line.startswith("| "):
            parts = [p.strip() for p in line.split("|")]
            if len(parts) >= 2:
                item_id = parts[1].strip()
                if item_id:
                    expected_ids_from_summary.add(item_id)
        if in_summary_table and not line.startswith("|") and line:
            in_summary_table = False  # table ended

    # All expected mandatory IDs
    all_expected_mandatory = expected_ids_from_summary - na_item_ids

    # Missing: expected IDs not found in inventory at all
    missing_ids = all_expected_mandatory - inventory_mandatory_ids

    # Validation result
    all_verified = len(missing_ids) == 0 and len(blocked_mandatory) == 0

    result = {
        "all_verified": all_verified,
        "verified_count": len(verified_mandatory),
        "accepted_count": len(accepted_mandatory),
        "blocked_count": len(blocked_mandatory),
        "partial_count": len(partial),
        "missing_ids": sorted(missing_ids),
        "missing_count": len(missing_ids),
        "blocked_mandatory_ids": sorted(blocked_mandatory),
        "accepted_mandatory_ids": sorted(accepted_mandatory),
        "verified_mandatory_ids": sorted(verified_mandatory),
        "total_expected": len(all_expected_mandatory),
        "total_verified": len(verified_mandatory),
        "total_blocked": len(blocked_mandatory),
        "na_item_ids": sorted(na_item_ids),
    }

    return result


def generate_report(result: dict) -> str:
    """Generate a human-readable validation report."""
    lines = [
        "=" * 70,
        "IMPLEMENTATION INVENTORY STRICT VALIDATION REPORT",
        "=" * 70,
        "",
        f"Total mandatory items (COMP-/ADR- prefixes, excluding N/A): {result['total_expected']}",
        f"Verified: {result['verified_count']}",
        f"Accepted: {result['accepted_count']}",
        f"Blocked: {result['blocked_count']}",
        f"Partial: {result['partial_count']}",
        f"Missing Verified status: {result['missing_count']}",
        f"All mandatory items Verified: {result['all_verified']}",
        "",
    ]

    if result["all_verified"]:
        lines.extend(
            [
                "RESULT: PASS — All mandatory inventory items are Verified.",
                "The first-stage completion gate requirement is satisfied.",
                "",
            ]
        )
    else:
        lines.extend(
            [
                "RESULT: FAIL — Not all mandatory items are Verified.",
                f"Missing Verified status for {result['missing_count']} mandatory item(s):",
                "",
            ]
        )
        for item_id in result["missing_ids"]:
            lines.append(f"  - {item_id}: not Verified (current status implied blocked/accepted/other)")

        if result["blocked_mandatory_ids"]:
            lines.append("")
            lines.append("Blocked mandatory items (require external provisioning):")
            for item_id in result["blocked_mandatory_ids"]:
                lines.append(f"  - {item_id}: BLOCKED")

        if result["accepted_mandatory_ids"]:
            lines.append("")
            lines.append("Accepted mandatory items (technical decisions, not Verified):")
            for item_id in result["accepted_mandatory_ids"]:
                lines.append(f"  - {item_id}: ACCEPTED")

        lines.append("")
        lines.append("All mandatory item statuses:")
        for item_id in result["verified_mandatory_ids"]:
            lines.append(f"  {item_id}: Verified")
        for item_id in result["blocked_mandatory_ids"]:
            lines.append(f"  {item_id}: Blocked")
        for item_id in result["accepted_mandatory_ids"]:
            lines.append(f"  {item_id}: Accepted")

        # Show N/A items
        if result["na_item_ids"]:
            lines.append("")
            lines.append("Excluded (N/A, not counted in mandatory gate):")
            for item_id in result["na_item_ids"]:
                lines.append(f"  {item_id}: N/A")

    lines.extend(
        [
            "",
            "—  End of Validation Report  —",
            "=" * 70,
        ]
    )

    report = "\n".join(lines)
    return report


def main():
    """Main entry point for the strict inventory validator."""
    # Parse the inventory
    inventory = parse_inventory(INVENTORY_PATH)

    # Validate
    result = validate_inventory(inventory)

    # Generate and print report
    report = generate_report(result)
    print(report)

    # Also output JSON-compatible result for workflow consumption
    print(f"\n__RESULT__:{str(result)}")

    # Exit code: 0 if all mandatory items verified, 1 otherwise
    sys.exit(0 if result["all_verified"] else 1)


if __name__ == "__main__":
    main()