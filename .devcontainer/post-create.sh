#!/usr/bin/env bash
set -e

# Install Python dependencies
uv sync
uv pip install -e .

# Install xdg-utils for Vault login
sudo apt-get update -y
sudo apt-get install -y --no-install-recommends xdg-utils
sudo rm -rf /var/lib/apt/lists/*

# Pull secrets from Vault
./scripts/secrets-setup.sh
./scripts/secrets-pull.sh
