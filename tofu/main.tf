terraform {
    backend "s3" {
        bucket = "tofu-state"
        key    = "governance/terraform.tfstate"
        region = "us-east-1"

        endpoints = {
            s3 = "https://s3.scottylabs.org"
        }

        skip_credentials_validation = true
        skip_requesting_account_id  = true
        skip_metadata_api_check     = true
        skip_region_validation      = true
        use_path_style              = true
    }

    required_providers {
        github = {
            source  = "integrations/github"
            version = "~> 6.0"
        }
        forgejo = {
            source  = "svalabs/forgejo"
            version = "~> 1.0"
        }
        keycloak = {
            source  = "keycloak/keycloak"
            version = "~> 5.0"
        }
        discord = {
            source  = "Lucky3028/discord"
            version = "~> 2.0"
        }
        google = {
            source  = "hashicorp/google"
            version = "~> 5.0"
        }
        bitwarden = {
            source  = "registry.terraform.io/maxlaverse/bitwarden"
            version = "~> 0.8"
        }
        restapi = {
            source  = "Mastercard/restapi"
            version = "~> 1.0"
        }
        random = {
            source  = "hashicorp/random"
            version = "~> 3.0"
        }
        external = {
            source  = "hashicorp/external"
            version = "~> 2.0"
        }
    }
}

resource "random_password" "kennel_webhook_secret" {
    length  = 64
    special = false
}

provider "github" {
    owner = var.github_org
    token = var.github_token
}

provider "forgejo" {
    host      = var.forgejo_url
    api_token = var.forgejo_token
}

provider "keycloak" {
    client_id     = var.keycloak_client_id
    client_secret = var.keycloak_client_secret
    url           = var.keycloak_url
    realm         = var.keycloak_realm
}

provider "discord" {
    token = var.discord_token
}

provider "google" {
    credentials           = var.google_credentials_json
    project               = var.google_project_id
    user_project_override = true
    billing_project       = var.google_project_id
}

provider "bitwarden" {
    server                = var.vaultwarden_url
    email                 = var.vaultwarden_email
    master_password       = var.vaultwarden_master_password
    client_implementation = "embedded"
    experimental {
        disable_sync_after_write_verification = true
    }
}

provider "restapi" {
    uri                    = var.forgejo_url
    write_returns_object   = true
    headers = {
        Authorization = "token ${var.forgejo_token}"
        Content-Type  = "application/json"
    }
}
