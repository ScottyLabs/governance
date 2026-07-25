# Forgejo action secrets for devops-troubleshooting-docs CI.
# The Garage key is created by the infrastructure/garage terranix config and
# written to Vault at shared/devops-docs-garage by that same apply.
# This config reads it from Vault and provisions the three secrets the
# .forgejo/workflows/deploy.yml workflow expects.

data "vault_kv_secret_v2" "devops_docs_garage" {
    mount = "secret"
    name  = "shared/devops-docs-garage"
}

resource "forgejo_repository_action_secret" "devops_docs_garage_access_key" {
    owner       = "ScottyLabs"
    repo        = "devops-troubleshooting-docs"
    secret_name = "GARAGE_ACCESS_KEY"
    data        = data.vault_kv_secret_v2.devops_docs_garage.data["access_key_id"]
}

resource "forgejo_repository_action_secret" "devops_docs_garage_secret_key" {
    owner       = "ScottyLabs"
    repo        = "devops-troubleshooting-docs"
    secret_name = "GARAGE_SECRET_KEY"
    data        = data.vault_kv_secret_v2.devops_docs_garage.data["secret_access_key"]
}

resource "forgejo_repository_action_secret" "devops_docs_garage_endpoint" {
    owner       = "ScottyLabs"
    repo        = "devops-troubleshooting-docs"
    secret_name = "GARAGE_ENDPOINT"
    data        = "https://s3.scottylabs.org"
}
