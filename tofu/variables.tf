variable "github_org" {
    type    = string
    default = "ScottyLabs"
}

variable "github_token" {
    type      = string
    sensitive = true
}

variable "forgejo_url" {
    type    = string
    default = "https://codeberg.org"
}

variable "forgejo_token" {
    type      = string
    sensitive = true
}

variable "keycloak_url" {
    type    = string
    default = "https://idp.scottylabs.org"
}

variable "keycloak_realm" {
    type    = string
    default = "scottylabs"
}

variable "keycloak_client_id" {
    type      = string
    sensitive = true
}

variable "keycloak_client_secret" {
    type      = string
    sensitive = true
}

variable "discord_token" {
    type      = string
    sensitive = true
}

variable "vaultwarden_url" {
    type    = string
    default = "https://vault.scottylabs.org"
}

variable "vaultwarden_email" {
    type      = string
    sensitive = true
}

variable "vaultwarden_master_password" {
    type      = string
    sensitive = true
}

variable "google_credentials_json" {
    type      = string
    sensitive = true
}

variable "google_project_id" {
    type    = string
    default = "sl-governance"
}

variable "vault_addr" {
    type    = string
    default = "https://secrets2.scottylabs.org"
}

variable "vault_approle_role_id" {
    type      = string
    sensitive = true
}

variable "vault_approle_secret_id" {
    type      = string
    sensitive = true
}

variable "kennel_webhook_url" {
    type    = string
    default = "https://kennel.scottylabs.org/webhook"
}

variable "docs_webhook_url" {
    type    = string
    default = "https://webhooks.scottylabs.org/hooks/docs-diagrams"
}

variable "sentry_organization" {
    type    = string
    default = "scottylabs"
}

variable "sentry_token" {
    type      = string
    sensitive = true
}

variable "sentry_base_url" {
    type    = string
    default = "https://sentry.io/api/"
}

variable "matrix_homeserver_url" {
    type    = string
    default = "https://matrix.doggylabs.org"
}

variable "matrix_domain" {
    type    = string
    default = "doggylabs.org"
}

variable "matrix_admin_token" {
    type      = string
    sensitive = true
}

variable "matrix_bridge_command_room_id" {
    type        = string
    default     = ""
    description = "Optional Matrix room ID for !discord create-portal when a portal is missing."
}

variable "matrix_slack_relay_login_id" {
    type        = string
    default     = ""
    description = "mautrix-slack relay login ID from `list-logins` after `login app` in @slack"
}
