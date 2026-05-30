#!/usr/bin/env bash
# Mirrors the generated observability CODEOWNERS file into
# ScottyLabs/observability when it has drifted.
#
# Inputs (env):
#   BOT_TOKEN    scottylabs-bot PAT with repo write
#   SIGNING_KEY  SSH signing key for commits
set -euo pipefail

# shellcheck source=lib/retry.sh
source "$(dirname "$0")/lib/retry.sh"

: "${BOT_TOKEN:?}"
: "${SIGNING_KEY:?}"

./target/release/governance observability-codeowners > /tmp/CODEOWNERS-new

retry 5 git clone "https://scottylabs-bot:${BOT_TOKEN}@codeberg.org/ScottyLabs/observability.git" /tmp/observability
mkdir -p /tmp/observability/.forgejo

if [ -f /tmp/observability/.forgejo/CODEOWNERS ] \
  && cmp -s /tmp/CODEOWNERS-new /tmp/observability/.forgejo/CODEOWNERS; then
  echo "observability CODEOWNERS already up to date"
  exit 0
fi

cp /tmp/CODEOWNERS-new /tmp/observability/.forgejo/CODEOWNERS

cd /tmp/observability
key=/tmp/signing_key
echo "$SIGNING_KEY" > "$key"
chmod 600 "$key"

git config user.name "scottylabs-bot"
git config user.email "ops+codeberg@scottylabs.org"
git config gpg.format ssh
git config user.signingkey "$key"
git config commit.gpgsign true

git add .forgejo/CODEOWNERS
git commit -m "chore: regenerate CODEOWNERS"
retry 5 git push
