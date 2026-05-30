#!/usr/bin/env bash
# Triggers a workflow_dispatch on the documentation hub deploy workflow.
#
# Inputs (env):
#   DOCS_TRIGGER_TOKEN  Codeberg token with write:repository on ScottyLabs/documentation
set -euo pipefail

# shellcheck source=lib/retry.sh
source "$(dirname "$0")/lib/retry.sh"

: "${DOCS_TRIGGER_TOKEN:?DOCS_TRIGGER_TOKEN required}"

retry 5 curl -fsS -X POST \
  -H "Authorization: token ${DOCS_TRIGGER_TOKEN}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  "https://codeberg.org/api/v1/repos/ScottyLabs/documentation/actions/workflows/deploy.yml/dispatches" \
  -d '{"ref":"main"}'
