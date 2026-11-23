#!/usr/bin/env python3
"""Regression tests for Omni's strict fixed-denominator inventory validator."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
VALIDATOR_PATH = ROOT / "ci" / "validate-inventory-strict.py"
SPEC = importlib.util.spec_from_file_location("omni_inventory_validator", VALIDATOR_PATH)
assert SPEC is not None and SPEC.loader is not None
VALIDATOR = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VALIDATOR
SPEC.loader.exec_module(VALIDATOR)


def make_inventory(status_overrides: dict[str, str] | None = None) -> str:
    statuses = {item_id: "Verified" for item_id in VALIDATOR.EXPECTED_GATE_IDS}
    statuses.update({
        "COMP-017": "N/A (duplicate of COMP-011; not a separate mandatory requirement)",
        "COMP-018": "N/A (identifier does not exist)",
    })
    if status_overrides:
        statuses.update(status_overrides)

    counts = {
        status: sum(statuses[item_id] == status for item_id in VALIDATOR.EXPECTED_GATE_IDS)
        for status in ("Verified", "Partial", "Accepted", "Blocked")
    }
    verified_listing = ", ".join(
        sorted(item_id for item_id in VALIDATOR.EXPECTED_GATE_IDS if statuses[item_id] == "Verified")
    )

    def rows_for(wanted: str, heading: str) -> str:
        del heading
        return "\n".join(
            f"| {item_id} | Item {item_id} | Path | Evidence | {wanted} |"
            for item_id in sorted(VALIDATOR.EXPECTED_GATE_IDS)
            if statuses[item_id] == wanted
        )

    verified_rows = rows_for("Verified", "Verified")
    partial_rows = rows_for("Partial", "Partial")
    accepted_rows = rows_for("Accepted", "Accepted")
    blocked_rows = rows_for("Blocked", "Blocked")
    excluded_rows = "\n".join(
        f"| {item_id} | Excluded | {statuses[item_id]} |"
        for item_id in sorted(VALIDATOR.EXCLUDED_IDS)
    )
    status_rows = "\n".join(
        f"| {item_id} | {statuses[item_id]} |" for item_id in sorted(statuses)
    )

    return f"""# Fixture

## Summary

| Category | Count | Items |
|---|---:|---|
| Verified | {counts['Verified']} | {verified_listing} |
| Partial | {counts['Partial']} | |
| Accepted | {counts['Accepted']} | |
| Blocked | {counts['Blocked']} | |
| **Total** | **26** | Mandatory |

## Verified Items (26)

| ID | Item | Path | Evidence | Status |
|---|---|---|---|---|
{verified_rows}

## Partial Items

| ID | Item | Path | Evidence | Status |
|---|---|---|---|---|
{partial_rows}

## Accepted Items

| ID | Item | Path | Status | Notes |
|---|---|---|---|---|
{accepted_rows}

## Blocked Items

| ID | Item | Path | Block Reason |
|---|---|---|---|
{blocked_rows}

## Excluded Identifiers

| ID | Item | Reason |
|---|---|---|
{excluded_rows}

## Requirement Identifier Status Summary

| ID | Status |
|---|---|
{status_rows}
"""


class StrictInventoryTests(unittest.TestCase):
    def write_inventory(self, content: str) -> tuple[tempfile.TemporaryDirectory, Path]:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        path = Path(temporary.name) / "IMPLEMENTATION_INVENTORY.md"
        path.write_text(content, encoding="utf-8")
        return temporary, path

    def test_exact_26_verified_records_pass_without_source_tree_check(self) -> None:
        _, path = self.write_inventory(make_inventory())
        result = VALIDATOR.validate_inventory(path)
        self.assertTrue(result["all_verified"], VALIDATOR.render_report(result))
        self.assertEqual(result["verified_count"], 26)
        self.assertEqual(result["accepted_ids"], [])
        self.assertEqual(result["blocked_ids"], [])

    def test_live_inventory_baseline_23_verified_one_partial_one_accepted_one_blocked_fails(self) -> None:
        _, path = self.write_inventory(make_inventory({
            "COMP-019": "Partial",
            "ADR-0002": "Accepted",
            "INST-002": "Blocked",
        }))
        result = VALIDATOR.validate_inventory(path)
        self.assertFalse(result["all_verified"])
        self.assertEqual(result["verified_count"], 23)
        self.assertEqual(result["partial_ids"], ["COMP-019"])
        self.assertEqual(result["accepted_ids"], ["ADR-0002"])
        self.assertEqual(result["blocked_ids"], ["INST-002"])

    def test_duplicate_comp017_must_remain_an_explicit_exclusion(self) -> None:
        content = make_inventory().replace(
            "N/A (duplicate of COMP-011; not a separate mandatory requirement)",
            "Verified",
        )
        _, path = self.write_inventory(content)
        result = VALIDATOR.validate_inventory(path)
        self.assertFalse(result["all_verified"])
        self.assertTrue(any("COMP-017 must have exclusion status" in error for error in result["errors"]))

    def test_missing_mandatory_record_fails(self) -> None:
        content = make_inventory().replace("| COMP-001 | Verified |\n", "")
        _, path = self.write_inventory(content)
        result = VALIDATOR.validate_inventory(path)
        self.assertFalse(result["all_verified"])
        self.assertTrue(any("COMP-001" in error for error in result["errors"]))

    def test_summary_count_mismatch_fails(self) -> None:
        content = make_inventory().replace("| Verified | 26 |", "| Verified | 25 |")
        _, path = self.write_inventory(content)
        result = VALIDATOR.validate_inventory(path)
        self.assertFalse(result["all_verified"])
        self.assertTrue(any("Summary Verified count" in error for error in result["errors"]))

    def test_live_repository_inventory_matches_current_evidence_classification(self) -> None:
        result = VALIDATOR.validate_inventory(ROOT / "IMPLEMENTATION_INVENTORY.md", ROOT)
        self.assertFalse(result["all_verified"])
        self.assertEqual(result["mandatory_total"], 26)
        self.assertEqual(result["verified_count"], 23)
        self.assertEqual(result["partial_ids"], ["COMP-019"])
        self.assertEqual(result["accepted_ids"], ["ADR-0002"])
        self.assertEqual(result["blocked_ids"], ["INST-002"])
        self.assertTrue(any("COMP-019 source evidence is not substantiated" in error for error in result["errors"]))

    def test_comp019_placeholder_sources_are_not_substantiated(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            inventory = root / "IMPLEMENTATION_INVENTORY.md"
            inventory.write_text(make_inventory(), encoding="utf-8")
            for relative in (
                "compiler/omni-pkg/src/main.rs",
                "compiler/omni-build/src/main.rs",
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("fn main() { // TODO: Implement\n}\n", encoding="utf-8")
            result = VALIDATOR.validate_inventory(inventory, root)
            self.assertFalse(result["all_verified"])
            self.assertTrue(any("COMP-019 source evidence" in error for error in result["errors"]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
