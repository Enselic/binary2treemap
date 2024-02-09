#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# CI sets this, so we should too
export CARGO_TERM_COLOR=always

cargo fmt -- --check

export RUSTDOCFLAGS='--deny warnings'
cargo doc \
    --locked \
    --no-deps \
    --document-private-items

cargo clippy \
    --locked \
    --all-targets \
    --all-features \
    -- \
    --deny clippy::all \
    --deny warnings \
    --deny unsafe_code

# Devs have to pretend to be a CI runner to opt-in to annoying but useful lints.
if [ "${CI:-}" = "true" ]; then
    cargo audit --deny warnings

    cargo deny check
fi
