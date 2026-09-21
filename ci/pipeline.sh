#!/bin/sh
set -eu

TIMEOUT_PR=1200
TIMEOUT_MAIN=1800
TIMEOUT_NIGHTLY=3600
TIMEOUT_QUAL=7200

MODE="${1:-pr}"

case "$MODE" in
    pr) MAX_TIME=$TIMEOUT_PR; FUZZ_BUDGET=60 ;;
    main) MAX_TIME=$TIMEOUT_MAIN; FUZZ_BUDGET=300 ;;
    nightly) MAX_TIME=$TIMEOUT_NIGHTLY; FUZZ_BUDGET=900 ;;
    qualification) MAX_TIME=$TIMEOUT_QUAL; FUZZ_BUDGET=1800 ;;
    *) echo "Unknown mode: $MODE" >&2; exit 1 ;;
esac

echo "CI mode=$MODE ceiling=${MAX_TIME}s fuzz_budget=${FUZZ_BUDGET}s/target"
# Low-memory guard: cap any Rust operation under ~500MB via single-job builds.
export CARGO_BUILD_JOBS=1
export CARGO_BUILD_CODEGEN_UNITS=1
./ci/assert-toolchain.sh

test "$(umask)" = "0022" || { echo "FATAL: umask must be 0022." >&2; exit 106; }

# The timeout command is part of the CI execution environment, not a language semantic dependency.
timeout "${MAX_TIME}s" cargo fmt --all -- --check
timeout "${MAX_TIME}s" cargo check -j 1 --workspace --locked
timeout "${MAX_TIME}s" cargo clippy -j 1 --workspace --all-targets --locked -- -D warnings
timeout "${MAX_TIME}s" cargo test -j 1 --workspace --locked -- --test-threads=1
timeout "${MAX_TIME}s" cargo metadata --format-version 1 --locked > /dev/null
timeout "${MAX_TIME}s" cargo run -j 1 -p omni-conform --locked
timeout "${MAX_TIME}s" cargo run -j 1 -p omni-audit --locked
# Fuzz budget enforcement (0.0.0.3): per-target budget declared above.
# Timeouts, hangs, crashes, sanitizer failures, and nondeterministic outputs are failures.
if command -v cargo-fuzz >/dev/null 2>&1; then
    echo "Fuzz: lexer_fuzz budget=${FUZZ_BUDGET}s"
    timeout "${MAX_TIME}s" cargo fuzz run lexer_fuzz -- -max_total_time="${FUZZ_BUDGET}"
else
    echo "WARN: cargo-fuzz not installed; fuzz budget (${FUZZ_BUDGET}s/target) declared but not enforced on this host."
fi
