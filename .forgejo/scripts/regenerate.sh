#!/usr/bin/env bash
# Regenerates tofu/, schemas/, and .forgejo/CODEOWNERS from data/.
set -euo pipefail

./target/debug/governance generate --output-dir tofu
./target/debug/governance schema --output-dir schemas
