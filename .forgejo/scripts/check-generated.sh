#!/usr/bin/env bash
# Drift check: regenerate into /tmp and diff against the in-tree copies.
# Fails if any committed generated file is stale.
set -euo pipefail

cp .forgejo/CODEOWNERS /tmp/CODEOWNERS-old
./target/debug/governance generate --output-dir /tmp/tofu
./target/debug/governance schema --output-dir /tmp/schemas
diff -r /tmp/tofu tofu/ --exclude='*.tf' --exclude='.gitignore'
diff -r /tmp/schemas schemas/
diff /tmp/CODEOWNERS-old .forgejo/CODEOWNERS
