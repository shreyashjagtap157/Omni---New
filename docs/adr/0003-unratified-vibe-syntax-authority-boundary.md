# ADR-0003: Unratified Vibe-first syntax and the Edition 1 authority boundary

Status: accepted (implementation decision; no normative syntax ratification).

Date: 2026-11-07.

## Decision

The default parser profile is strict Edition 1. Unratified Vibe-first syntax is
available only through an explicit `ExperimentalVibeSyntax` parser profile
for isolated research and regression testing. The profile is not a ratified
language edition and does not make Candidate 2 or Candidate 3 normative.

The production driver and public compile/execute entry points always select
Edition 1. Experimental driver execution remains test-only until an authorized
candidate profile and end-to-end conformance contract exist.

The Edition 1 precedence function excludes the candidate pipeline operator.
A separately named experimental precedence function is used only by the
experimental parser profile.

Command-style calls remain disabled. Optional navigation is recognized only by
the experimental parser and remains unsupported by driver lowering. Candidate-2
newline termination and semicolon changes remain disabled and unimplemented.

This decision changes compiler profile selection and tests only. It does not
modify the normative grammar, manifest, registry lifecycle states, release
digest, or Candidate 2/3 ratification status.

## Consequences

- Default language and production driver behavior cannot accidentally treat
  unratified Vibe-first syntax as Edition 1.
- Parser and abstract-machine experiments remain available under explicit
  opt-in/test-only paths.
- Experimental parser acceptance is not a compiler conformance result.
- Future candidate promotion still requires the full governance, semantics,
  grammar, diagnostics, lowering, compatibility and qualification process.
## Context

The master development mandate requires the in-force Edition 1 grammar to remain
authoritative, forbids activating Candidate 2, and prohibits implementation
from inventing or adopting semantics from pending proposals. The repository
currently contains conflicting records and observable parser behavior.

### Repository evidence at decision time

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
6. `compiler/omni-parse/src/parser.rs` parses `PipelineExpr`,
   `QuestionDotExpr`, and prefix-dot `FieldExpr` nodes used for projection
   shorthand. In `compiler/omni-driver/src/main.rs`, `expr_from_node` lowers
   pipeline expressions to calls/method calls and prefix-dot field expressions
   to lambdas. The driver has a `CommandCallExpr` lowering arm, but the parser
   initializes `allow_command_call = false`, no source assignment enables it,
   and a parser test explicitly rejects out-of-force command-call syntax.
   `QuestionDotExpr` has no driver lowering arm and falls through to the
   unsupported-node error path. Driver tests execute pipeline lowering on the
   abstract machine; the grammar-contract test also accepts `a |> b;` and
   `a?.b;` through the default parser entry point.
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

## Decision record (resolved)

The accepted decision is Alternative A: the default parser is strict Edition 1,
and selected parser experiments remain available only through an explicit,
non-normative research profile. This decision has been implemented in the
parser and production driver. It does not ratify a candidate amendment.

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

## Adopted option

Alternative A is the adopted implementation boundary: strict Edition 1 is the
production default; selected candidate syntax is available only through the
explicit `ExperimentalVibeSyntax` parser profile. The parser and driver change
is implemented and validated. Any future change to the normative EBNF,
candidate ratification, compatibility policy, or release manifest still
requires the controlled specification-governance process.

The implementation decision does not remove or ratify syntax. It changes only
the default parser profile and production driver behavior.

## Current implementation state

- `Parser::from_source` selects `ParserProfile::Edition1`. It rejects pipeline
  `|>`, optional navigation `?.`, prefix-dot projection shorthand, and
  standalone expression-placeholder `_` in expression position.
- `Parser::from_source_with_profile(..., ParserProfile::ExperimentalVibeSyntax)`
  opts into the limited parser experiments. Acceptance under this profile is
  not Edition 1 conformance and is not Candidate-2/3 ratification.
- The production driver and public compile/execute entry points use Edition 1.
  Experimental pipeline execution remains test-only.
- Optional navigation remains unsupported by driver lowering; command-call
  syntax remains disabled.
- Candidate-2 newline termination and optional semicolons remain disabled and
  unimplemented.
- The trailing VIBE-GRAM block in the normative EBNF remains an open document
  defect. This ADR does not edit the grammar, rebind its digest, or change
  registry lifecycle states.

## Further approval boundaries

No additional approval is required for the accepted parser-profile
implementation recorded here. Separate explicit approval remains required to
ratify Candidate 2/3, alter normative grammar or semantics, change
compatibility obligations, or rebind the release manifest/version.

This ADR leaves the normative Edition 1 grammar, manifest digest, and registry
lifecycle states unchanged. Its accepted implementation decision **does**
change parser behavior and production driver defaults as described above.
