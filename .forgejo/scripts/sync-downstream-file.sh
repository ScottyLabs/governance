#!/usr/bin/env bash
# Mirrors a governance-generated file into a downstream ScottyLabs repo when it
# has drifted, committing as scottylabs-bot.
#
# Usage: sync-downstream-file.sh <generate-command> <repo> <path-in-repo> <label>
#
# Inputs (env):
#   BOT_TOKEN    scottylabs-bot PAT with repo write
#   SIGNING_KEY  SSH signing key for commits
set -euo pipefail

# shellcheck source=lib/retry.sh
source "$(dirname "$0")/lib/retry.sh"

: "${BOT_TOKEN:?}"
: "${SIGNING_KEY:?}"

cmd="${1:?generate command}"
repo="${2:?downstream repo}"
dest="${3:?path in repo}"
label="${4:?file label}"

work="/tmp/$repo"
new="/tmp/sync-new"

./target/debug/governance "$cmd" > "$new"

retry 5 git clone "https://scottylabs-bot:${BOT_TOKEN}@codeberg.org/ScottyLabs/${repo}.git" "$work"
mkdir -p "$(dirname "$work/$dest")"

if [ -f "$work/$dest" ] && cmp -s "$new" "$work/$dest"; then
  echo "$repo $label already up to date"
  exit 0
fi

cp "$new" "$work/$dest"

cd "$work"
key=/tmp/signing_key
echo "$SIGNING_KEY" > "$key"
chmod 600 "$key"

git config user.name "scottylabs-bot"
git config user.email "ops+codeberg@scottylabs.org"
git config gpg.format ssh
git config user.signingkey "$key"
git config commit.gpgsign true

git add "$dest"
git commit -m "chore: regenerate $label"
retry 5 git push
