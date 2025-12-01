#!/usr/bin/env bash
export VAULT_ADDR=https://secrets.scottylabs.org

cat .env | xargs -r vault kv put -mount="ScottyLabs" "governance"
