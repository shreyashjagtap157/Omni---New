# ADR-0002: Foundation-gate technical evidence and formal qualification

Status: proposed (not accepted).

Date: 2026-11-07.

## Context

The master development mandate, §68, identifies a conflict between the
repository's foundation-gate records and explicitly prohibits inferring
foundation qualification merely from a committed JSON status field. It also
states that qualification of the foundation gate must not be confused with
qualification of the complete compiler or language.

The current repository state contains several distinct kinds of evidence:

1. `spec/release/foundation-gate.json` declares
   `OMNI-IMP-0.0.0.13` as `qualified`, with an empty `blocked_reasons`
   array and specification digest
   `3986c2a71d9e5e4092331d025079055f34b98aa4179de1684b64e28bde74e6ed`.
2. `docs/foundation-gates.md` still describes 0.0.0.13 as
   `pre-sign-off; no gate declared` and contains intermediate/historical
   digest values. Its current-digest statements are inconsistent with the
   manifest and the canonical tool output.
3. The full Foundation 0.0.0.13 CI job on commit
   `a84389630e0746953137f2b86d65e2e5e7d15299` passed its ten technical
   steps. Its log records the canonical digest above, 567 registry rules,
   zero evidence records, and 14 allowed plus 8 forbidden Stage-0 predicates.
   This is technical CI evidence for that commit, not by itself a formal
   sign-off or a product-release qualification.
4. `omni-conform` deliberately accepts an absent or empty evidence corpus
   vacuously. The current run therefore verifies the evidence-validation
   mechanism but does not demonstrate rule-level witness coverage.
5. The current registry contains 567 Candidate rules and no Ratified rules.
   The documentation also records zero emitted E-code diagnostics and zero
   rule-witness records. `omni-audit` can pass while witness obligations are
   vacuous because they attach to Ratified rules.
6. The historical Security Audit on commit `a843896` failed on
   RUSTSEC-2026-0344 (`ruzstd 0.7.3`). The unused direct `object`
   dependency was removed in `d270e938c64543310f5672070075dd92cfd6691f`,
   eliminating the compression feature path that brought the vulnerable
   decoder into the lockfile. Cargo Audit run `42364182354` on descendant
   commit `c81c413961d7ec403d58899ebd7d0bdff443145b` completed successfully:
   zero vulnerabilities, zero warnings. Later commits through current HEAD
   `78e23db8` change documentation/ADR/changelog only; `Cargo.toml` and
   `Cargo.lock` remain unchanged. A fresh audit for commit `4b929084` is
   still running at the time of this update.
7. `omni-conform` now requires the declared gate status to be `qualified`
   and `blocked_reasons` to be empty. This validates declaration consistency,
   not independent approval or formal qualification. Full Omni CI on
   `64f734b97a22ab562a97bacf36979391b763504d` and
   `c81c413961d7ec403d58899ebd7d0bdff443145b` passed, including the new
   negative gate-status/blocker tests and the Foundation technical gate.
   The latest full CI run on `4b929084` has passed formatting, focused
   ownership tests, Clippy, workspace check, workspace tests, locked metadata,
   and manifest binding; topology is in progress and the Foundation gate is
   pending at the time of this update.

These facts must not be collapsed into a single green checkmark.

## Decision to be made

Define what `OMNI-IMP-0.0.0.13` means and what evidence is required before
it can be called formally qualified.

In particular, decide whether the foundation gate qualifies:

- only the deterministic repository/tooling/specification infrastructure;
- that infrastructure plus the presence of a functioning evidence mechanism;
- or that infrastructure plus a minimum amount of real rule-level conformance
  evidence and independent review.

The decision must also define whether a human approval/sign-off is required,
who may provide it, and how that approval is bound to the exact qualified
commit without creating a self-referential digest.

## Alternatives

### A. Technical foundation qualification only

Treat the ten-step CI gate as sufficient for the narrowly scoped foundation
milestone. The empty evidence corpus and Candidate-only rule registry are
explicitly outside this milestone's acceptance criteria.

Consequences:
- The gate may be called technically qualified once all required technical
  checks pass on the exact commit.
- Documentation must state that no complete compiler, normative rule corpus,
  or language release is thereby qualified.
- The evidence corpus remains a separately tracked future obligation.

Risk: readers may interpret the word `qualified` more broadly than intended
unless the scope is made explicit everywhere.

### B. Pre-sign-off until rule-level evidence and independent approval exist

Keep the foundation milestone pre-sign-off until the required diagnostics,
witnesses, conformance outcomes and reviewer approval exist.

Consequences:
- The gate remains closed to downstream qualification.
- The gate record and documentation must agree on the pre-sign-off state.
- The exact minimum evidence threshold and applicable rules must be specified.

Risk: this may impose rule-level evidence obligations earlier than the
foundation milestone intended, so the requirement must be grounded in the
approved roadmap rather than invented.

### C. Separate technical-gate outcome from formal qualification

Represent technical CI outcome and formal qualification as separate states.
A technical run can pass while formal sign-off remains pending. Formal
qualification is backed by a separate, commit-bound attestation or review
record with explicit evidence references and approval identity.

Consequences:
- CI cannot turn a self-declared JSON status into a formal qualification.
- Technical failures, unresolved evidence obligations, and approval status
  remain independently visible.
- The foundation milestone can be technically ready without being formally
  signed off.
- The same distinction can later be reused for platform and product releases.

Cost: requires a small qualification-record schema, validation logic, and
review workflow.

## Recommendation

Prefer Alternative C.

It most directly satisfies the master mandate's requirement not to infer
qualification from a JSON status field, while avoiding the opposite mistake
of treating every empty evidence corpus as an automatic failure without first
establishing that rule-level witnesses are required at this milestone.

The existing status/blocker validation in `omni-conform` should be treated
only as a consistency guard. It is necessary, not sufficient, evidence of
qualification.

## Interim handling (until this ADR is accepted)

- Do not describe the complete Omni compiler, language, or a product release
  as qualified on the basis of the foundation gate.
- Describe the 0.0.0.13 technical CI checks by their exact commit and run.
- Do not silently change the current gate status, required evidence threshold,
  or release/version policy.
- Keep the digest discrepancy visible until `docs/foundation-gates.md` is
  reconciled with the canonical manifest and tool output.
- Re-run all checks, including Security Audit, on the latest `main` commit
  before making a current qualification claim.
- Preserve the existing historical records.

## Approval required

The project owner/maintainer must approve the milestone scope, minimum
evidence threshold, and sign-off authority before Alternative A, B, or C is
adopted as normative qualification policy.

No normative language grammar or semantics are changed by this proposal.
