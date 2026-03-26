terraform {
    required_providers {
        github = {
            source  = "integrations/github"
            version = "~> 6.0"
        }
        forgejo = {
            source  = "svalabs/forgejo"
            version = "~> 0.2"
        }
        keycloak = {
            source  = "keycloak/keycloak"
            version = "~> 5.0"
        }
        discord = {
            source  = "spaceshuttl/discord"
            version = "~> 0.1"
        }
        slack = {
            source  = "jmatsu/slack"
            version = "~> 0.3"
        }
        google = {
            source  = "hashicorp/google"
            version = "~> 5.0"
        }
        bitwarden = {
            source  = "maxlaverse/bitwarden"
            version = "~> 0.8"
        }
        restapi = {
            source  = "Mastercard/restapi"
            version = "~> 1.0"
        }
        tls = {
            source  = "hashicorp/tls"
            version = "~> 4.0"
        }
        external = {
            source  = "hashicorp/external"
            version = "~> 2.0"
        }
    }
}

provider "github" {
    owner = var.github_org
    token = var.github_token
}

provider "forgejo" {
    host  = var.forgejo_url
    token = var.forgejo_token
}

provider "keycloak" {
    client_id     = var.keycloak_client_id
    client_secret = var.keycloak_client_secret
    url           = var.keycloak_url
    realm         = var.keycloak_realm
}

provider "restapi" {
    uri = var.forgejo_url
    headers = {
        Authorization = "token ${var.forgejo_token}"
        Content-Type  = "application/json"
    }
}
