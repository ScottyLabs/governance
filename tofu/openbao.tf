data "vault_auth_backend" "oidc" {
    path = "oidc"
}

# DevOps team vault access
resource "vault_policy" "devops" {
    name   = "devops"
    policy = <<-EOT
        # Manage all secrets
        path "secret/*" {
            capabilities = ["create", "read", "update", "delete", "list"]
        }

        # Manage AppRole auth
        path "auth/approle/*" {
            capabilities = ["create", "read", "update", "delete", "list"]
        }

        # View OIDC auth
        path "auth/oidc/*" {
            capabilities = ["read", "list"]
        }

        # View policies
        path "sys/policies/*" {
            capabilities = ["read", "list"]
        }

        # View auth methods
        path "sys/auth" {
            capabilities = ["read"]
        }
    EOT
}

resource "vault_identity_group" "devops" {
    name     = "devops"
    type     = "external"
    policies = [vault_policy.devops.name]
}

resource "vault_identity_group_alias" "devops" {
    name           = "/projects/devops"
    mount_accessor = data.vault_auth_backend.oidc.accessor
    canonical_id   = vault_identity_group.devops.id
}
