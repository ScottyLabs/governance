{
  terraform = {
    required_providers = {
      github = {
        source = "integrations/github";
        version = "~> 6.0";
      };
      forgejo = {
        source = "svalabs/forgejo";
        version = "~> 1.0";
      };
      keycloak = {
        source = "keycloak/keycloak";
        version = "~> 5.0";
      };
      discord = {
        source = "Lucky3028/discord";
        version = "~> 2.0";
      };
      google = {
        source = "hashicorp/google";
        version = "~> 5.0";
      };
      bitwarden = {
        source = "registry.terraform.io/maxlaverse/bitwarden";
        version = "~> 0.8";
      };
      restapi = {
        source = "Mastercard/restapi";
        version = "~> 1.0";
      };
      vault = {
        source = "hashicorp/vault";
        version = "~> 5.0";
      };
      random = {
        source = "hashicorp/random";
        version = "~> 3.0";
      };
      sentry = {
        source = "jianyuan/sentry";
        version = "~> 0.14.0";
      };
      posthog = {
        source = "PostHog/posthog";
        version = "~> 1.0";
      };
      external = {
        source = "hashicorp/external";
        version = "~> 2.0";
      };
      synapse = {
        source = "thesuperrl/synapse";
        version = "0.2.0";
      };
      garage = {
        source = "registry.terraform.io/jkossis/garage";
        version = "~> 1.0";
      };
      litellm = {
        source = "registry.terraform.io/ncecere/litellm";
        version = "~> 2.0";
      };
    };

    backend.s3 = {
      bucket = "tofu-state";
      key = "governance/terraform.tfstate";
      region = "us-east-1";
      endpoints.s3 = "https://s3.scottylabs.org";
      skip_credentials_validation = true;
      skip_requesting_account_id = true;
      skip_metadata_api_check = true;
      skip_region_validation = true;
      use_path_style = true;
    };
  };

  provider = {
    vault = {
      address = "\${var.vault_addr}";
      auth_login = {
        path = "auth/approle/login";
        parameters = {
          role_id = "\${var.vault_approle_role_id}";
          secret_id = "\${var.vault_approle_secret_id}";
        };
      };
    };
    litellm = {
      api_base = "\${var.litellm_url}";
      api_key = "\${data.vault_kv_secret_v2.litellm_master_key.data[\"MASTER_KEY\"]}";
    };
    github = {
      owner = "\${var.github_org}";
      token = "\${var.github_token}";
    };
    forgejo = {
      host = "\${var.forgejo_url}";
      api_token = "\${var.forgejo_token}";
    };
    keycloak = {
      client_id = "\${var.keycloak_client_id}";
      client_secret = "\${var.keycloak_client_secret}";
      url = "\${var.keycloak_url}";
      realm = "\${var.keycloak_realm}";
    };
    discord.token = "\${var.discord_token}";
    google = {
      credentials = "\${var.google_credentials_json}";
      project = "\${var.google_project_id}";
      user_project_override = true;
      billing_project = "\${var.google_project_id}";
    };
    bitwarden = {
      server = "\${var.vaultwarden_url}";
      email = "\${var.vaultwarden_email}";
      master_password = "\${var.vaultwarden_master_password}";
      client_implementation = "embedded";
      experimental.disable_sync_after_write_verification = true;
    };
    restapi = {
      uri = "\${var.forgejo_url}";
      write_returns_object = true;
      headers = {
        Authorization = "token \${var.forgejo_token}";
        "Content-Type" = "application/json";
      };
    };
    sentry = {
      token = "\${var.sentry_token}";
      base_url = "\${var.sentry_base_url}";
    };
    posthog = {
      api_key = "\${var.posthog_token}";
      host = "\${var.posthog_host}";
      organization_id = "\${var.posthog_organization_id}";
    };
    synapse = {
      homeserver_url = "\${var.matrix_homeserver_url}";
      domain = "\${var.matrix_domain}";
      admin_token = "\${var.matrix_admin_token}";
      bridge_command_room_id = "\${var.matrix_bridge_command_room_id}";
      slack_team_id = "\${try(local.matrix_slack_team_id, \"\")}";
      slack_relay_login_id = "\${var.matrix_slack_relay_login_id}";
    };
    garage = {
      endpoint = "\${var.garage_admin_endpoint}";
      token = "\${var.garage_admin_token}";
    };
  };

  variable = {
    github_org = {
      type = "string";
      default = "ScottyLabs";
    };
    github_token = {
      type = "string";
      sensitive = true;
    };
    forgejo_url = {
      type = "string";
      default = "https://codeberg.org";
    };
    forgejo_token = {
      type = "string";
      sensitive = true;
    };
    keycloak_url = {
      type = "string";
      default = "https://idp.scottylabs.org";
    };
    keycloak_realm = {
      type = "string";
      default = "scottylabs";
    };
    keycloak_client_id = {
      type = "string";
      sensitive = true;
    };
    keycloak_client_secret = {
      type = "string";
      sensitive = true;
    };
    discord_token = {
      type = "string";
      sensitive = true;
    };
    vaultwarden_url = {
      type = "string";
      default = "https://vault.scottylabs.org";
    };
    vaultwarden_email = {
      type = "string";
      sensitive = true;
    };
    vaultwarden_master_password = {
      type = "string";
      sensitive = true;
    };
    google_credentials_json = {
      type = "string";
      sensitive = true;
    };
    google_project_id = {
      type = "string";
      default = "sl-governance";
    };
    vault_addr = {
      type = "string";
      default = "https://secrets.scottylabs.org";
    };
    vault_approle_role_id = {
      type = "string";
      sensitive = true;
    };
    vault_approle_secret_id = {
      type = "string";
      sensitive = true;
    };
    kennel_webhook_url = {
      type = "string";
      default = "https://kennel.scottylabs.org/webhook";
    };
    docs_webhook_url = {
      type = "string";
      default = "https://webhooks.scottylabs.org/hooks/docs-diagrams";
    };
    sentry_organization = {
      type = "string";
      default = "scottylabs";
    };
    sentry_token = {
      type = "string";
      sensitive = true;
    };
    sentry_base_url = {
      type = "string";
      default = "https://sentry.io/api/";
    };
    posthog_token = {
      type = "string";
      sensitive = true;
    };
    posthog_host = {
      type = "string";
      default = "https://us.posthog.com";
    };
    posthog_organization_id = {
      type = "string";
    };
    matrix_homeserver_url = {
      type = "string";
      default = "https://matrix.doggylabs.org";
    };
    matrix_domain = {
      type = "string";
      default = "doggylabs.org";
    };
    matrix_admin_token = {
      type = "string";
      sensitive = true;
    };
    matrix_bridge_command_room_id = {
      type = "string";
      default = "";
      description = "Optional Matrix room ID for !discord create-portal when a portal is missing.";
    };
    matrix_slack_relay_login_id = {
      type = "string";
      default = "";
      description = "mautrix-slack relay login ID from `list-logins` after `login app` in @slack";
    };
    garage_admin_endpoint = {
      type = "string";
      default = "http://127.0.0.1:3903";
    };
    garage_admin_token = {
      type = "string";
      sensitive = true;
    };
    garage_s3_endpoint = {
      type = "string";
      default = "https://s3.scottylabs.org";
    };
    cdn_base_url = {
      type = "string";
      default = "https://cdn.scottylabs.org";
    };
    litellm_url = {
      type = "string";
      default = "https://litellm.scottylabs.org";
    };
  };

  data = {
    vault_kv_secret_v2 = {
      litellm_master_key = {
        mount = "secret";
        name = "infra/litellm-master-key";
      };
      cachix = {
        mount = "secret";
        name = "shared/cachix";
      };
      sccache = {
        mount = "secret";
        name = "shared/sccache";
      };
    };
    keycloak_realm.this.realm = "scottylabs";
    forgejo_organization.this.name = "ScottyLabs";
    vault_auth_backend.oidc.path = "oidc";
  };

  resource = {
    keycloak_group.projects = {
      realm_id = "\${data.keycloak_realm.this.id}";
      name = "projects";
    };

    forgejo_organization_action_secret = {
      cachix_auth_token = {
        organization_id = "\${data.forgejo_organization.this.id}";
        name = "CACHIX_AUTH_TOKEN";
        data = "\${data.vault_kv_secret_v2.cachix.data[\"CACHIX_AUTH_TOKEN\"]}";
      };
      sccache_access_key_id = {
        organization_id = "\${data.forgejo_organization.this.id}";
        name = "AWS_ACCESS_KEY_ID";
        data = "\${data.vault_kv_secret_v2.sccache.data[\"AWS_ACCESS_KEY_ID\"]}";
      };
      sccache_secret_access_key = {
        organization_id = "\${data.forgejo_organization.this.id}";
        name = "AWS_SECRET_ACCESS_KEY";
        data = "\${data.vault_kv_secret_v2.sccache.data[\"AWS_SECRET_ACCESS_KEY\"]}";
      };
    };

    vault_policy = {
      devops = {
        name = "devops";
        policy = ''
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
        '';
      };
      shared_read = {
        name = "shared-read";
        policy = ''
          path "secret/data/shared/*" {
              capabilities = ["read"]
          }
        '';
      };
    };

    vault_identity_group = {
      devops = {
        name = "devops";
        type = "external";
        policies = [ "\${vault_policy.devops.name}" ];
      };
      all_members = {
        name = "all-members";
        type = "external";
        policies = [ "\${vault_policy.shared_read.name}" ];
      };
    };

    vault_identity_group_alias = {
      devops = {
        name = "/projects/devops";
        mount_accessor = "\${data.vault_auth_backend.oidc.accessor}";
        canonical_id = "\${vault_identity_group.devops.id}";
      };
      all_members = {
        name = "/projects";
        mount_accessor = "\${data.vault_auth_backend.oidc.accessor}";
        canonical_id = "\${vault_identity_group.all_members.id}";
      };
    };

    random_password.kennel_webhook_secret = {
      length = 64;
      special = false;
    };
  };
}
