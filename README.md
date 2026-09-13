# Omni

Omni is the implementation repository for the Omni Edition 1 language and toolchain.

This repository is governed by the normative specification under `spec/` and the implementation master plan recorded at repository initialization. Implementation semantics must not be invented locally when the specification is silent or contradictory.

The implementation proceeds through three generational stages:

- Stage 1: Rust-only Minimal Working Omni Core (`0.0.0.1` → `1.0.0.0`)
- Stage 2: Rust-only Complete Production Omni Platform (`1.0.0.1` → `2.0.0.0`)
- Stage 3: Omni-written, self-hosted, independently qualified Omni (`2.0.0.1` → `3.0.0.0`)
