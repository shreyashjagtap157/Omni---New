# Normative erratum: Candidate-2 surface amendment and the grammar publication

This record is a normative erratum overlay in the sense of REL-0007
(rule-linked; effective releases, replacement scope, and impact stated
below). It amends nothing by reinterpretation: every normative statement
below is excerpted or cited from the authoritative Candidate 2 amendment,
which carries status "Candidate amendment; not ratified and not
implementation-certified".

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
instead of leaving two authorities in conflict.

## Rule links

`LEX-0002`, `GRAM-0001`, `GRAM-0002`, `GRAM-0003`, `VIBE-GRAM-0001`,
`VIBE-GRAM-0002`, `VIBE-GRAM-0003`, `VIBE-GRAM-0004`, `VIBE-GRAM-0005`,
`VIBE-GRAM-0006`, `VIBE-GRAM-0007`, `RULE-0003`, `REL-0003`, `REL-0007`.

## Status and limits (amendment appendix, sections P, Q, R)

The Candidate 2 surface is reported as planned/candidate, never complete,
until the referenced lexer, parser, formatter, desugaring, diagnostics,
semantic, and conformance gates pass. The conformance corpus for the
amended surface constructs and the promotion of Candidate 2 to a ratified
baseline (including a machine-readable unambiguous grammar) remain future
work. This erratum does not promote Candidate 2 and does not alter any
RULE-0003 lifecycle state.
