#!/usr/bin/env bash
# Regenerates tofu/, schemas/, and .forgejo/CODEOWNERS from data/.
set -euo pipefail

cargo run -q -p governance -- generate --output-dir tofu
cargo run -q -p governance -- schema --output-dir schemas
