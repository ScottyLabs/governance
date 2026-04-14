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

variable "kennel_webhook_url" {
    type    = string
    default = "https://kennel.scottylabs.org/webhook"
}
