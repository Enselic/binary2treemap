#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# CI sets this, so we should too
export CARGO_TERM_COLOR=always

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

./scripts/lint.sh

cargo test --locked
