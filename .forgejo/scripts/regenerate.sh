#!/usr/bin/env bash
# Regenerates tofu/, schemas/, and .forgejo/CODEOWNERS from data/.
set -euo pipefail

./target/release/governance generate --output-dir tofu
./target/release/governance schema --output-dir schemas
