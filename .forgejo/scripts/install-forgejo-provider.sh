#!/usr/bin/env bash
# Installs the patched Forgejo Terraform provider via a dev_overrides config
# and exports TF_CLI_CONFIG_FILE to $GITHUB_ENV so later steps pick it up.
set -euo pipefail

readonly PROVIDER_URL="https://github.com/ap-1/terraform-provider-forgejo/releases/download/v1.4.2-team-repository.2/terraform-provider-forgejo_1.4.2-team-repository.2_linux_amd64.zip"
readonly PROVIDER_DIR="/tmp/forgejo-provider"

mkdir -p "$PROVIDER_DIR"
curl -sL "$PROVIDER_URL" -o /tmp/fp.zip
unzip -o /tmp/fp.zip -d "$PROVIDER_DIR"
chmod +x "$PROVIDER_DIR"/terraform-provider-forgejo_*

mkdir -p "$HOME/.terraform.d"
cat > "$HOME/.terraform.d/dev_overrides.tfrc" <<EOF
provider_installation {
  dev_overrides {
    "svalabs/forgejo" = "$PROVIDER_DIR"
  }
  direct {}
}
EOF

echo "TF_CLI_CONFIG_FILE=$HOME/.terraform.d/dev_overrides.tfrc" >> "$GITHUB_ENV"
