# ADR-0003: Unratified Vibe-first syntax and the Edition 1 authority boundary

Status: proposed (not accepted).

Date: 2026-11-07.

## Context

The master development mandate requires the in-force Edition 1 grammar to remain
authoritative, forbids activating Candidate 2, and prohibits implementation
from inventing or adopting semantics from pending proposals. The repository
currently contains conflicting records and observable parser behavior.

### Repository evidence

1. `spec/grammar/candidate2-erratum.md` labels the Candidate-2 overlay
   `pre-release`, `signature_state: pending`, and
   `implementation_impact: none`. It explicitly says the overlay is not
   signed or applied and that the Candidate-1 baseline continues to govern.
2. `docs/grammar-reconciliation.md` says Candidate-2 newline termination,
   optional semicolons, pipeline promotion, projection shorthand and
   command-call disambiguation are not in force. It says the Candidate-2
   feature gate is not enabled.
3. `spec/registry/rules.json` records `VIBE-GRAM-0001` through the related
   VIBE-GRAM rules with lifecycle status `Candidate`, not `Ratified`.
4. `spec/grammar/omni-edition1.ebnf` contains a trailing section labelled
   `VIBE-GRAM extensions – normatively required productions that the baseline
   EBNF omitted`, defining `pipeline_operator`, `newline_termination`,
   `projection_shorthand` and `command_call`. These bytes are inside the
   canonical specification-tree digest.
5. `compiler/omni-parse/src/grammar_contract.rs` includes
   `candidate3_pipeline_and_optional_navigation_parse`, which accepts
   `a |> b;` and `a?.b;` while the same module asserts strict Edition 1
   semicolon/newline behavior.
6. `compiler/omni-parse/src/parser.rs` has explicit parsing paths for
   `QuestionDotExpr`, projection shorthand in pipeline stage arguments,
   and `CommandCallExpr`. The driver contains corresponding lowering logic.
   These paths are not selected through an explicit Candidate-2/3 profile
   gate in the evidence reviewed for this proposal.
7. The legacy `IMPLEMENTATION_PLAN.md` contains `ERR3-0030` through
   `ERR3-0037` implementation statuses, including pipeline, projection,
   command calls and optional chaining. `IMPLEMENTATION_PLAN_GENERATED.md`
   separately says Candidate 2 and Candidate 3 are not ratified,
   not implementation-certified, and are documentation-only.
8. The current normative specification tree digest is
   `3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`.
   Any change to the in-force EBNF requires the controlled specification
   process and canonical digest rebinding.

This means the parser's observable acceptance, the normative grammar artifact,
the erratum's declared implementation impact, the registry lifecycle, and the
implementation plans do not currently establish one unambiguous authority.

## Decision to be made

Determine which surface syntax is authorized in the default Edition 1 parser
and whether any Candidate-2/3 experiments may remain available behind an
explicit non-normative profile.

The decision must distinguish:

- syntax defined by the in-force Edition 1 EBNF;
- Candidate-2 amendment syntax covered by the unsigned pre-release overlay;
- Candidate-3 or ERR3 proposal syntax;
- parser experiments that currently accept syntax without a ratified rule;
- syntax that may be lexed as reserved punctuation but must not be accepted as
  a valid Edition 1 construct.

## Alternatives

### A. Strict Edition 1 default; candidate experiments isolated

Make the default parser accept only the in-force Edition 1 contract. Candidate
syntax is rejected in that mode. Any future experimental mode must be explicit,
separately named, non-default, and must not emit Edition 1 conformance claims.

Consequences:
- Prevents accidental Candidate-2/3 activation.
- Requires negative parser tests for every candidate-only construct.
- Requires auditing lexer, parser, lowering, type checking and driver entry
  points so a candidate feature cannot leak through a later phase.
- Does not decide whether a future candidate should be ratified.

### B. Ratify a specific candidate amendment before implementation

Select Candidate 2 or Candidate 3 through the governed process, establish
rule lifecycle changes and signed replacement grammar, rebind the specification
manifest, define migration and compatibility, then implement the full surface
and its conformance corpus.

Consequences:
- Makes the accepted syntax normative and testable.
- Requires explicit approval and a complete amendment package.
- Must not be inferred from partial parser support or an implementation-plan
  status label.

### C. Remove candidate syntax from the normative EBNF and parser

Restore the Candidate-1 grammar artifact and parser behavior, preserving the
candidate proposals only as separate documentation.

Consequences:
- Strongest immediate alignment with the Candidate-1 baseline.
- Potentially breaking for programs that currently use the candidate syntax.
- Requires exact historical preservation and a controlled specification
  digest/version migration.

## Recommendation

Prefer Alternative A as the target architecture: strict Edition 1 behavior is
the default, and candidate experiments cannot affect conformance claims. The
precise migration from the current tree must be approved through the
specification-governance process before changing the normative EBNF or deleting
existing implementation paths.

No syntax is removed, added, promoted, or newly ratified by this proposal.

## Interim handling

- Treat the observed pipeline, optional-navigation, projection and command-call
  paths as unqualified experimental behavior until their authority is resolved.
- Do not claim that their acceptance establishes Edition 1 conformance.
- Do not enable Candidate 2 or Candidate 3 by default.
- Preserve current grammar bytes and historical records until the approved
  decision determines whether to restore Candidate-1 grammar or ratify a
  replacement.
- Continue independent implementation work on unambiguous Edition 1
  productions and compiler subsystems.
- Update the implementation plan only after its status claims are reconciled
  against the normative registry and actual source/tests.

## Approval required

The project owner/maintainer must select the intended candidate baseline,
approve the default parser profile, and authorize any normative grammar,
semantic, compatibility, or release-manifest changes.

This proposal does not modify the normative Edition 1 grammar, manifest,
registry lifecycle states, parser behavior, or product version.
