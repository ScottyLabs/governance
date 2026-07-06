data "forgejo_organization" "this" {
    name = "ScottyLabs"
}

data "vault_kv_secret_v2" "cachix" {
    mount = "secret"
    name  = "shared/cachix"
}

data "vault_kv_secret_v2" "sccache" {
    mount = "secret"
    name  = "shared/sccache"
}

# Org-level Actions secrets the shared CI workflow forwards via `secrets: inherit`
resource "forgejo_organization_action_secret" "cachix_auth_token" {
    organization_id = data.forgejo_organization.this.id
    name            = "CACHIX_AUTH_TOKEN"
    data            = data.vault_kv_secret_v2.cachix.data["CACHIX_AUTH_TOKEN"]
}

resource "forgejo_organization_action_secret" "sccache_access_key_id" {
    organization_id = data.forgejo_organization.this.id
    name            = "AWS_ACCESS_KEY_ID"
    data            = data.vault_kv_secret_v2.sccache.data["AWS_ACCESS_KEY_ID"]
}

resource "forgejo_organization_action_secret" "sccache_secret_access_key" {
    organization_id = data.forgejo_organization.this.id
    name            = "AWS_SECRET_ACCESS_KEY"
    data            = data.vault_kv_secret_v2.sccache.data["AWS_SECRET_ACCESS_KEY"]
}
