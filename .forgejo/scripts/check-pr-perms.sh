#!/usr/bin/env bash
# Validates that the PR author is allowed to modify the files changed against
# the base branch.
#
# Inputs (env):
#   BASE_REF   base branch name (e.g. "main")
#   PR_AUTHOR  PR submitter's Codeberg login
set -euo pipefail

# shellcheck source=lib/retry.sh
source "$(dirname "$0")/lib/retry.sh"

: "${BASE_REF:?BASE_REF required}"
: "${PR_AUTHOR:?PR_AUTHOR required}"

retry 5 git fetch origin "$BASE_REF"
changed=$(git diff --name-only "origin/${BASE_REF}...HEAD" | tr '\n' ',')

./target/release/governance check-pr \
  --author "$PR_AUTHOR" \
  --base-ref "origin/${BASE_REF}" \
  --changed-files "$changed"
