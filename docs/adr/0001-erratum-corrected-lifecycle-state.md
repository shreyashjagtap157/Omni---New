# ADR-0001: `ErratumCorrected` lifecycle state spelling and ownership

Status: accepted.

## Context

RULE-0003 defines seven lifecycle states: proposed, candidate, ratified,
deprecated, superseded, withdrawn, and erratum-corrected. The machine-readable
registry schema and every validator enforced only six, omitting the seventh.
A spelling mismatch between normative authority and machine-readable
schema/registry is an authority-boundary defect: a record carrying the seventh
state would fail validation despite being normatively legitimate.

REL-0003 distinguishes a corrigendum (publication error, no intended semantics
change) from a normative erratum (corrects semantics, declares compatibility
impact). REL-0007 requires erratum overlays to be rule-linked. Nothing in this
record changes those rules; it only fixes the missing enum spelling.

## Decision

The machine-readable spelling is `ErratumCorrected`: capitalized single token,
consistent with the other six enum spellings (`Proposed`, `Candidate`,
`Ratified`, `Deprecated`, `Superseded`, `Withdrawn`). The hyphenated lowercase
form remains exclusively in normative prose.

Ownership: implementation claims on `ErratumCorrected` records are forbidden,
exactly like `Deprecated`, `Superseded`, and `Withdrawn`. An erratum-corrected
record is history: the correction lives in the linked overlay, and
implementations target the current revision. Witness obligations attach only
to `Ratified` records (unchanged).

## Consequences

- `spec/schemas/rule-registry.schema.json` enum gains `ErratumCorrected`.
- The canonical seven-state set lives once in `omni-registry`
  (`LIFECYCLE_STATES`) with ownability helpers; `omni-audit`,
  `omni-conform`, and `omni-evidence` consume it instead of local copies.
- A test asserts the schema file's enum equals the canonical set, so the
  normative text, schema, registry data, loader, and validators agree
  mechanically rather than by parallel editing.
- No existing record changes state: the registry holds zero
  `ErratumCorrected` records today, which is itself asserted to keep the
  vocabulary honest (present but unpopulated, not silently dropped).
