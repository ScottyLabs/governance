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
