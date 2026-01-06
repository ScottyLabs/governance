#!/usr/bin/env bash
export VAULT_ADDR=https://secrets.scottylabs.org

vault kv get -format=json ScottyLabs/governance |
jq -r '.data.data | to_entries[] | "\(.key)=\"\(.value | gsub("\n"; "\\n"))\""' >.env
echo "Pulled from vault: ScottyLabs/governance"
