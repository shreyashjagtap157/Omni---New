# Goal: align-omni-parse-driver (superseded)

**Status: STALE — do not execute as an implementation plan.**

## Reason for supersession

This record asks for alignment with an alleged “Candidate 3” specification and requires changing the Edition 1 manifest to `candidate: 3` and `status: released`. The live normative contract does not support those requirements:

- `spec/grammar/omni-edition1.ebnf` explicitly identifies itself as the normative grammar for Omni Edition 1.
- `spec/manifest/omni-edition1.manifest.json` identifies edition 1 and status `1.0.0-candidate.1`; it does not authorize Candidate 3.
- `spec/grammar/candidate2-erratum.md` and `docs/grammar-reconciliation.md` record the Candidate 2 overlay as pre-release/pending and not in force. Candidate 2 remains disabled.
- The Candidate 3 acceptance criteria in the original goal conflict with the above sources and cannot override normative specification artifacts.

## Disposition

Keep this file as a historical record of a superseded request, not as an active target. Do not change the Edition 1 grammar, registry, manifest, hashes, or release metadata to satisfy this goal. Do not introduce Candidate 3 syntax or revive the historical Candidate 2 `|>` projection/pipeline surface.

Any future parser/driver work must be independently grounded in the active Edition 1 grammar and applicable normative rules, with parser/driver regression tests and the repository's pinned qualification workflow. If a future ratified edition or signed amendment changes the authority boundary, first reconcile the specification artifacts and release identity through the project's explicit qualification process.

## Retired acceptance criteria (historical only)

The superseded request asked to resolve 13 parser/driver failures against Candidate 3, update the grammar/manifest to Candidate 3 released, preserve MIR/ownership/codegen behaviour, and run workspace tests. These are not current acceptance criteria because their asserted specification authority is unsupported.
