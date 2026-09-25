# REL-0007 erratum overlay (pre-release): Candidate-2 surface amendment

This record is a **pre-release normative erratum overlay** in the sense of
`REL-0007`. It declares every element that rule requires, in the machine block
below, and it declares the one element the release process has not yet
established: signature. It is therefore published, rule-linked, and immutably
bound, but **not signed and not applied** to any release. Nothing here claims a
completion the repository has not established.

Two concepts stay separate and are never merged by this record:

```text
Candidate-2 amendment  ≠  ratified Edition-1 grammar
```

Every normative statement below is excerpted or cited from the authoritative
Candidate 2 amendment, which itself carries status "Candidate amendment; not
ratified and not implementation-certified". This overlay does not promote it.

## Machine-readable metadata (REL-0007 elements; authoritative)

```rel-0007
{
  "overlay_id": "erratum-candidate2-surface",
  "classification": "rel-0007-erratum-overlay",
  "status": "pre-release",
  "effective_releases": ["1.0.0-candidate.1"],
  "replacement_source_path": "docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.2_vibe.md",
  "replacement_source_sha256": "e9725ebc12ee1d67d6d97c7924d4ff36e036dff1fd89eb207564e0022c65fb8d",
  "implementation_impact": "none: no implementation consumes this overlay; the Candidate-1 baseline and grammar/omni-edition1.ebnf continue to govern every observable behavior",
  "migration": "none while pre-release; on ratification, migrate the superseded-rule behaviors listed below under replacement material and re-run the conformance corpus",
  "immutability": "spec-tree-sha256",
  "signature_state": "pending"
}
```

## REL-0007 element status

| Element | Requirement | State of this overlay |
| --- | --- | --- |
| Rule-linked | cites the rules it affects | Satisfied: rule links below; unresolved citations fail the loader |
| Immutable | cannot be changed silently | Satisfied: bytes are inside the specification-tree digest bound by the manifest and the release gate, so any edit rebinds the digest and fails closed |
| Effective releases | names where it takes effect | Declared: `1.0.0-candidate.1`, the status of the base manifest that lists this file |
| Replacement text/data/tests | states what it replaces | Declared by exact path and SHA-256 of the replacement source, plus the superseded and introduced rule identities below; no replacement text is restated or paraphrased here |
| Implementation impact | states impact on implementations | Declared: none, because no implementation consumes this overlay yet |
| Migration | states how affected implementations move | Declared: none while pre-release; on ratification, migrate the superseded-rule behaviors named below |
| Signed | carries a signature | **Not established**: no signing key exists at `0.0.0.x`, so `signature_state` stays `pending`, `status` stays `pre-release`, and the loader rejects any attempt to mark the overlay `applied` |

## Supersession scope (amendment appendix, sections A and C)

This amendment supersedes earlier Candidate.1 wording only where this
section explicitly states a surface-syntax change. The explicitly stated
changes are:

- `LEX-0002`, `GRAM-0002`, and `GRAM-0003` are superseded for ordinary
  source statements by `VIBE-GRAM-0001`, `VIBE-GRAM-0002`,
  `VIBE-GRAM-0003`, `VIBE-GRAM-0004`, `VIBE-GRAM-0005`,
  `VIBE-GRAM-0006`, and `VIBE-GRAM-0007`.
- `|>` is promoted from reserved punctuation to a standard pipeline
  operator (the reservation itself was unnumbered prose, as is the
  promotion; no rule identity is created or destroyed by either).

## Precedence rule

Within the stated scope, the appendix governs. Outside it, the
Candidate-1 baseline governs, including `grammar/omni-edition1.ebnf` as
named by `GRAM-0001`. Where the two readings conflict inside the stated
scope, the appendix reading is the conforming one; no conforming
interpretation applies both. This resolves the co-normative disagreement
instead of leaving two authorities in conflict, and it does so without
changing the status of either authority: `ROOT-0002` still forbids any
implementation from choosing between them.

## Rule links

`LEX-0002`, `GRAM-0001`, `GRAM-0002`, `GRAM-0003`, `VIBE-GRAM-0001`,
`VIBE-GRAM-0002`, `VIBE-GRAM-0003`, `VIBE-GRAM-0004`, `VIBE-GRAM-0005`,
`VIBE-GRAM-0006`, `VIBE-GRAM-0007`, `RULE-0003`, `REL-0003`, `REL-0007`,
`REL-0008`, `ROOT-0001`, `ROOT-0002`.

## Status and limits (amendment appendix, sections P, Q, R)

The Candidate 2 surface is reported as planned/candidate, never complete,
until the referenced lexer, parser, formatter, desugaring, diagnostics,
semantic, and conformance gates pass. The conformance corpus for the
amended surface constructs and the promotion of Candidate 2 to a ratified
baseline (including a machine-readable unambiguous grammar) remain future
work. This erratum does not promote Candidate 2, does not alter any
`RULE-0003` lifecycle state, and does not assert `REL-0007` completion:
signature and application are outstanding, and `REL-0008` corrected claims
remain unavailable until both are established.
