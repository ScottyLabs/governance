#!/usr/bin/env bash
# Installs local-only OpenTofu providers for CI until they are on the registry:
#   - patched Forgejo (temporary; team-repository support is being upstreamed)
#   - thesuperrl/synapse (fork of https://gitlab.com/risson/terraform-provider-synapse)
set -euo pipefail

readonly FORGEJO_URL="https://github.com/ap-1/terraform-provider-forgejo/releases/download/v1.4.2-team-repository.2/terraform-provider-forgejo_1.4.2-team-repository.2_linux_amd64.zip"
readonly FORGEJO_DIR="/tmp/forgejo-provider"
readonly SYNAPSE_DIR="/tmp/synapse-provider"
readonly SYNAPSE_VERSION="0.2.0"
readonly SYNAPSE_REPO_URL="${SYNAPSE_PROVIDER_REPO_URL:-https://codeberg.org/thesuperRL/terraform-provider-synapse.git}"
readonly SYNAPSE_SRC_CACHE="/tmp/terraform-provider-synapse-src"
readonly REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

case "$(uname -m)" in
  x86_64) TARGET=linux_amd64 ;;
  aarch64 | arm64) TARGET=linux_arm64 ;;
  *)
    echo "unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

# Patched Forgejo binary (dev_overrides replaces registry download at runtime).
mkdir -p "$FORGEJO_DIR"
curl -fsSL "$FORGEJO_URL" -o /tmp/forgejo-provider.zip
unzip -o /tmp/forgejo-provider.zip -d "$FORGEJO_DIR"
chmod +x "$FORGEJO_DIR"/terraform-provider-forgejo_*

synapse_provider_src() {
  if [ -n "${SYNAPSE_PROVIDER_SRC:-}" ] && [ -f "${SYNAPSE_PROVIDER_SRC}/go.mod" ]; then
    echo "${SYNAPSE_PROVIDER_SRC}"
    return
  fi
  local sibling="${REPO_ROOT}/../terraform-provider-synapse"
  if [ -f "${sibling}/go.mod" ]; then
    echo "${sibling}"
    return
  fi
  if [ ! -f "${SYNAPSE_SRC_CACHE}/go.mod" ]; then
    rm -rf "${SYNAPSE_SRC_CACHE}"
    git clone --depth 1 "${SYNAPSE_REPO_URL}" "${SYNAPSE_SRC_CACHE}"
  fi
  echo "${SYNAPSE_SRC_CACHE}"
}

if ! command -v go >/dev/null; then
  GO_VERSION="${GO_VERSION:-1.23.6}"
  GOROOT="${HOME}/.local/go"
  case "$(uname -m)" in
    x86_64) GOARCH=amd64 ;;
    aarch64 | arm64) GOARCH=arm64 ;;
    *)
      echo "unsupported architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac
  echo "Installing Go ${GO_VERSION} (${GOARCH}) for synapse provider build..."
  curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-${GOARCH}.tar.gz" -o /tmp/go.tar.gz
  rm -rf "${GOROOT}"
  mkdir -p "${HOME}/.local"
  tar -C "${HOME}/.local" -xzf /tmp/go.tar.gz
  export PATH="${GOROOT}/bin:${PATH}"
fi

SYNAPSE_PROVIDER_SRC="$(synapse_provider_src)"
mkdir -p "$SYNAPSE_DIR"
(
  cd "$SYNAPSE_PROVIDER_SRC"
  CGO_ENABLED=0 go build -mod=readonly -o "$SYNAPSE_DIR/terraform-provider-synapse" .
)

SYNAPSE_PLUGIN_ROOT="${HOME}/.terraform.d/plugins/registry.opentofu.org/thesuperrl/synapse/${SYNAPSE_VERSION}/${TARGET}"
mkdir -p "$SYNAPSE_PLUGIN_ROOT"
install -m 755 "$SYNAPSE_DIR/terraform-provider-synapse" \
  "$SYNAPSE_PLUGIN_ROOT/terraform-provider-synapse_v${SYNAPSE_VERSION}"

mkdir -p "${HOME}/.terraform.d"
cat > "${HOME}/.terraform.d/dev_overrides.tfrc" <<EOF
provider_installation {
  dev_overrides {
    "svalabs/forgejo" = "$FORGEJO_DIR"
  }
  filesystem_mirror {
    path = "${HOME}/.terraform.d/plugins"
  }
  direct {}
}
EOF

if [ -n "${GITHUB_ENV:-}" ]; then
  echo "TF_CLI_CONFIG_FILE=${HOME}/.terraform.d/dev_overrides.tfrc" >> "$GITHUB_ENV"
fi
echo "Installed Forgejo override at $FORGEJO_DIR"
echo "Installed synapse provider at $SYNAPSE_PLUGIN_ROOT (from ${SYNAPSE_PROVIDER_SRC})"
