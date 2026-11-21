# ADR-0002: Foundation Gate Sign-Off - Decision Package

## Status: Accepted (Alternative C)

## Date: 2026-11-19

## Approved By: Project Owner (owner authorization, November 19, 2026)

## Decision

**Alternative C approved: Separate technical-gate outcome from formal qualification.**

Technical CI results and formal qualification are independently recorded. Passing CI does not constitute governance approval. Formal qualification requires a separate commit-bound attestation with explicit evidence references and approval identity.

## Policy

1. Technical CI results and formal qualification are independently recorded.
2. Passing CI does not constitute governance approval.
3. An empty evidence corpus must be disclosed explicitly.
4. Candidate-only registry status must be disclosed explicitly.
5. Rule-level conformance obligations remain independently tracked.
6. Foundation qualification cannot be interpreted as complete compiler, complete language or product-release qualification.
7. Formal approval must reference an exact source commit and its qualification evidence.
8. The qualification record must not introduce a self-referential specification digest.
9. A technical reviewer must independently examine the qualification package.
10. The project owner/maintainer is the approval authority, unless an independently established repository governance rule explicitly assigns that authority elsewhere.

## Implementation

See `spec/release/qualification-record.schema.json` for the qualification record schema.
See `compiler/omni-conform/` for conformance validation logic.
See `docs/foundation-gates.md` for updated documentation.

## Current State

- `spec/release/foundation-gate.json`: `OMNI-IMP-0.0.0.13` declared `"status": "qualified"` with empty `blocked_reasons`
- This is a TECHNICAL declaration, not formal sign-off
- Formal qualification: PENDING owner-attested record
- 567 registry rules: all Candidate (zero Ratified)
- Evidence corpus: empty (zero witness records, zero E-code diagnostics)

## Authority Classification

**VERIFIED** - Alternative C approved by project owner. Implementation of qualification record schema and validation in progress.
