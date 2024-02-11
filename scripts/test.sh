#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

cargo test --locked
