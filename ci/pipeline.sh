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
./ci/assert-toolchain.sh

test "$(umask)" = "0022" || { echo "FATAL: umask must be 0022." >&2; exit 106; }

# The timeout command is part of the CI execution environment, not a language semantic dependency.
timeout "${MAX_TIME}s" cargo fmt --all -- --check
timeout "${MAX_TIME}s" cargo check --workspace --locked
timeout "${MAX_TIME}s" cargo clippy --workspace --all-targets --locked -- -D warnings
timeout "${MAX_TIME}s" cargo test --workspace --locked
timeout "${MAX_TIME}s" cargo metadata --format-version 1 --locked > /dev/null
