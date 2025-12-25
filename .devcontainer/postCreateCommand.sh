#!/usr/bin/env bash
pip3 install -r __meta/synchronizer/requirements.txt

# Install xdg-utils for Vault login
sudo apt-get update -y
sudo apt-get install -y --no-install-recommends xdg-utils
sudo rm -rf /var/lib/apt/lists/*

# Pull secrets from Vault
./scripts/secrets-setup.sh
./scripts/secrets-pull.sh
