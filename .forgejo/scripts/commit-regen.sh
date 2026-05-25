#!/usr/bin/env bash
# Commits regenerated tofu/schemas/CODEOWNERS back to the PR head branch as
# scottylabs-bot if they drifted from the PR's committed copies. No-op when
# already in sync. Fails for fork PRs without "Allow edits from maintainers".
#
# Inputs (env):
#   BOT_TOKEN       scottylabs-bot PAT with repo write
#   SIGNING_KEY     SSH private key used to sign the commit
#   HEAD_CLONE_URL  PR head clone URL (https)
#   HEAD_FULL       PR head repo full_name (owner/repo)
#   BASE_FULL       PR base repo full_name (owner/repo)
#   HEAD_REF        PR head branch name
#   ALLOW_EDIT      "true" if maintainers may edit the PR
set -euo pipefail

: "${BOT_TOKEN:?}"
: "${SIGNING_KEY:?}"
: "${HEAD_CLONE_URL:?}"
: "${HEAD_FULL:?}"
: "${BASE_FULL:?}"
: "${HEAD_REF:?}"
: "${ALLOW_EDIT:?}"

if git diff --quiet -- tofu schemas .forgejo/CODEOWNERS; then
  echo "Generated files already in sync"
  exit 0
fi

if [ "$HEAD_FULL" != "$BASE_FULL" ] && [ "$ALLOW_EDIT" != "true" ]; then
  echo "::error::tofu/schemas/CODEOWNERS need regeneration but this fork PR does not have 'Allow edits from maintainers' enabled."
  echo "::error::Enable that option on the PR, or run 'governance generate' + 'governance schema' locally and push the result."
  git --no-pager diff --stat -- tofu schemas .forgejo/CODEOWNERS
  exit 1
fi

key=/tmp/signing_key
echo "$SIGNING_KEY" > "$key"
chmod 600 "$key"

git config user.name "scottylabs-bot"
git config user.email "ops+codeberg@scottylabs.org"
git config gpg.format ssh
git config user.signingkey "$key"
git config commit.gpgsign true

git add tofu schemas .forgejo/CODEOWNERS
git commit -m "chore: regenerate tofu, schemas, CODEOWNERS"

push_url="${HEAD_CLONE_URL/https:\/\//https://scottylabs-bot:${BOT_TOKEN}@}"
git push "$push_url" "HEAD:refs/heads/${HEAD_REF}"
