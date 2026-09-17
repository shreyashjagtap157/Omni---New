# Omni Edition 1 Candidate 2 Vibe-First — Complete Handoff

## Repository state

Current local branch: `main`

Current HEAD: `88c0aeb02d656d0f96114e9f7096582373c0f87f`

Parent: `67e0763a28d801a4615432a1af9f7b0c2c393718`

Historical baseline immediately before this documentation/amendment work: `1c97ac39b34738e2a43ae018362a9f22ee344380`

The current working tree is clean.

## Candidate status

Candidate.1 remains the existing normative baseline manifest.

Candidate.2 Vibe-First is a controlled candidate/amendment overlay. It is not represented as ratified language, and it is not implementation-certified merely because the documents exist.

Candidate.2 changes only the explicitly amended surface-syntax contract. Existing semantic guarantees remain authoritative.

## Candidate 2 surface target

The language is deliberately low-ceremony and AI-friendly, with deterministic grammar and explicit semantic desugaring.

Representative forms:

```omni
fn greet(name) {
    print "Hello, {name}!"
}

users
    |> where .active
    |> map .name
    |> sort
    |> take 20

with file = open(path)? {
    process file
}

parallel {
    profile = fetch_profile(user)
    posts = fetch_posts(user)
}

fn send_report(report)
    requires Email.Send, Storage.Read
    -> Result<(), Error> / io
{
    ...
}
```

## Mandatory implementation architecture

```text
source
-> lossless lexer
-> parser events
-> lossless CST
-> surface AST
-> deterministic vibe desugaring
-> semantic AST/HIR
-> names
-> types
-> ownership/lifetimes
-> effects/capabilities
-> MIR
-> verifier
-> reference machine/backend
```

Do not implement vibe syntax as arbitrary natural-language interpretation.

## Immediate engineering work

1. Inspect current lexer/parser/CST/AST and the normative grammar.
2. Reconcile the Candidate.1 newline/semicolon rules with Candidate.2.
3. Add the Candidate.2 token vocabulary, including pipeline syntax.
4. Implement deterministic newline/continuation handling.
5. Implement pipeline parsing and precedence.
6. Implement projection shorthand.
7. Implement deterministic command-style calls.
8. Implement optional chaining/coalescing where missing.
9. Implement surface AST nodes.
10. Implement deterministic surface-to-core lowering.
11. Propagate source spans and semantic identity through desugaring.
12. Integrate with names, types, ownership, effects, capabilities, and concurrency.
13. Update formatter/layout behavior.
14. Build the complete syntax/conformance corpus.
15. Run the full verification suite.
16. Update implementation/status/evidence docs.
17. Commit a coherent milestone.
18. Push to the authoritative remote.
19. Verify local and remote commit IDs.
20. Continue upward through the master implementation plan.

## Required evidence per construct

Each construct needs:

- lexical coverage;
- valid parser fixture;
- invalid parser fixture;
- recovery fixture where applicable;
- CST golden;
- source round-trip test where losslessness is required;
- surface AST coverage;
- deterministic desugaring golden;
- semantic acceptance test;
- semantic rejection test where applicable;
- formatter test;
- regression test;
- documentation.

## Semantic non-regression

Surface syntax must never bypass:

- type checking;
- ownership or borrowing;
- lifetime validation;
- effect checking;
- capability authorization;
- unsafe obligations;
- cancellation semantics;
- FFI boundaries;
- comptime isolation;
- distributed-state restrictions;
- reproducibility rules.

## Qualification rule

Candidate.2 is implementation-qualified only after the parser, CST, AST, desugaring, formatter, semantic integrations, conformance fixtures, diagnostics, reproducibility checks, and applicable project-wide gates pass.

## Verification environment note

The generated handoff bundle has been validated as a complete Git history bundle. The current documentation-generation environment does not have `cargo` installed, so Rust build/test/clippy qualification was not independently rerun here. Treat prior build reports as claims to be independently reproduced by the implementation agent.

## Git bundle

The associated all-ref bundle is generated with `git bundle create --all` and verified with `git bundle verify`.
