#!/usr/bin/env bash
# Validates that the PR author is allowed to modify the files changed against
# the base branch. Files only touched by scottylabs-bot regen commits on the
# PR head are excluded.
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

mapfile -t all_changed < <(git diff --name-only "origin/${BASE_REF}...HEAD")

human_changed=()
for f in "${all_changed[@]}"; do
  if git log "origin/${BASE_REF}..HEAD" --pretty=format:'%an' -- "$f" \
    | grep -qvFx scottylabs-bot; then
    human_changed+=("$f")
  fi
done

if [ ${#human_changed[@]} -eq 0 ]; then
  echo "PR diff contains only bot-authored changes; nothing to check"
  exit 0
fi

changed=$(IFS=','; echo "${human_changed[*]}")

./target/release/governance check-pr \
  --author "$PR_AUTHOR" \
  --base-ref "origin/${BASE_REF}" \
  --changed-files "$changed"
