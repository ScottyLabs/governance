#!/usr/bin/env bash
uv sync --project __meta/synchronizer/pyproject.toml

# Install xdg-utils for Vault login
sudo apt-get update -y
sudo apt-get install -y --no-install-recommends xdg-utils
sudo rm -rf /var/lib/apt/lists/*

# Pull secrets from Vault
./scripts/secrets-setup.sh
./scripts/secrets-pull.sh
