#!/bin/bash
set -e

PR_AUTHOR=$1
TEAM_FILE=$2

# Skip non-TOML files
if [[ "$TEAM_FILE" != *.toml ]]; then
    echo "Skipping non-TOML file: $TEAM_FILE"
    exit 0
fi

# Check if PR author is in members list
if ! grep -qE "members\s*=\s*\[" "$TEAM_FILE" || ! grep -qE "(\"$PR_AUTHOR\"|'$PR_AUTHOR')" "$TEAM_FILE"; then
    echo "::error::Team creator must be listed as a member in $TEAM_FILE"
    exit 1
fi

echo "Team creator membership validated"
