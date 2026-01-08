#!/usr/bin/env bash
set -e

# Install Python dependencies
uv sync
uv pip install -e .

# Install xdg-utils for Vault login
sudo apt-get update -y
sudo apt-get install -y --no-install-recommends xdg-utils
sudo rm -rf /var/lib/apt/lists/*

# Create alias in uv lint
uvlint_alias="alias uvlint='uv run ruff check && uv run mypy . && uv run ty check'"
if ! grep -q "$uvlint_alias" ~/.zshrc; then
  echo "$uvlint_alias" >>~/.zshrc
fi

# Pull secrets from Vault
./scripts/secrets/setup.sh
./scripts/secrets/pull.sh
