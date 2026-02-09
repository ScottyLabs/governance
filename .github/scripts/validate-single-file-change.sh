#!/bin/bash
set -euo pipefail

DIR_PREFIX="$1"
CHANGED_FILES="$2"

COUNT=0

for FILE in $CHANGED_FILES; do
  if [[ "$FILE" == "$DIR_PREFIX"* ]]; then
    COUNT=$((COUNT + 1))
  fi
done

if [[ "$COUNT" -gt 1 ]]; then
  echo "::error::Only one file may be added or modified in '$DIR_PREFIX' per PR"
  exit 1
fi

echo "Validated: only one file changed in $DIR_PREFIX"
