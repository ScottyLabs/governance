use serde_json::{Map, Value, json};

use crate::tf_json::TfJsonFile;

fn backend_s3(axis: &str) -> Value {
    json!({
        "bucket": "tofu-state",
        "key": format!("governance/{axis}.tfstate"),
        "region": "us-east-1",
        "endpoints": { "s3": "https://s3.scottylabs.org" },
        "skip_credentials_validation": true,
        "skip_requesting_account_id": true,
        "skip_metadata_api_check": true,
        "skip_region_validation": true,
        "use_path_style": true,
    })
}

fn required_provider(name: &str) -> Value {
    match name {
        "keycloak" => json!({ "source": "keycloak/keycloak", "version": "~> 5.0" }),
        "forgejo" => json!({ "source": "svalabs/forgejo", "version": "~> 1.0" }),
        "github" => json!({ "source": "integrations/github", "version": "~> 6.0" }),
        "restapi" => json!({ "source": "Mastercard/restapi", "version": "~> 1.0" }),
        "random" => json!({ "source": "hashicorp/random", "version": "~> 3.0" }),
        "null" => json!({ "source": "hashicorp/null", "version": "~> 3.0" }),
        "external" => json!({ "source": "hashicorp/external", "version": "~> 2.0" }),
        "vault" => json!({ "source": "hashicorp/vault", "version": "~> 5.0" }),
        "litellm" => {
            json!({ "source": "registry.terraform.io/ncecere/litellm", "version": "~> 2.0" })
        }
        "discord" => json!({ "source": "Lucky3028/discord", "version": "~> 2.0" }),
        "google" => json!({ "source": "hashicorp/google", "version": "~> 5.0" }),
        "bitwarden" => {
            json!({ "source": "registry.terraform.io/maxlaverse/bitwarden", "version": "~> 0.8" })
        }
        "sentry" => json!({ "source": "jianyuan/sentry", "version": "~> 0.14.0" }),
        "posthog" => json!({ "source": "PostHog/posthog", "version": "~> 1.0" }),
        "synapse" => json!({ "source": "thesuperrl/synapse", "version": "0.2.0" }),
        "garage" => {
            json!({ "source": "registry.terraform.io/jkossis/garage", "version": "~> 1.0" })
        }
        other => panic!("unknown provider: {other}"),
    }
}

fn provider_config(name: &str) -> Option<Value> {
    Some(match name {
        "keycloak" => json!({
            "client_id": "${var.keycloak_client_id}",
            "client_secret": "${var.keycloak_client_secret}",
            "url": "${var.keycloak_url}",
            "realm": "${var.keycloak_realm}",
        }),
        "forgejo" => json!({
            "host": "${var.forgejo_url}",
            "api_token": "${var.forgejo_token}",
        }),
        "github" => json!({
            "owner": "${var.github_org}",
            "token": "${var.github_token}",
        }),
        "restapi" => json!({
            "uri": "${var.forgejo_url}",
            "write_returns_object": true,
            "headers": {
                "Authorization": "token ${var.forgejo_token}",
                "Content-Type": "application/json",
            },
        }),
        "vault" => json!({
            "address": "${var.vault_addr}",
            "auth_login": {
                "path": "auth/approle/login",
                "parameters": {
                    "role_id": "${var.vault_approle_role_id}",
                    "secret_id": "${var.vault_approle_secret_id}",
                },
            },
        }),
        "litellm" => json!({
            "api_base": "${var.litellm_url}",
            "api_key": "${data.vault_kv_secret_v2.litellm_master_key.data[\"MASTER_KEY\"]}",
        }),
        "discord" => json!({ "token": "${var.discord_token}" }),
        "google" => json!({
            "credentials": "${var.google_credentials_json}",
            "project": "${var.google_project_id}",
            "user_project_override": true,
            "billing_project": "${var.google_project_id}",
        }),
        "bitwarden" => json!({
            "server": "${var.vaultwarden_url}",
            "email": "${var.vaultwarden_email}",
            "master_password": "${var.vaultwarden_master_password}",
            "client_implementation": "embedded",
            "experimental": { "disable_sync_after_write_verification": true },
        }),
        "sentry" => json!({
            "token": "${var.sentry_token}",
            "base_url": "${var.sentry_base_url}",
        }),
        "posthog" => json!({
            "api_key": "${var.posthog_token}",
            "host": "${var.posthog_host}",
            "organization_id": "${var.posthog_organization_id}",
        }),
        "synapse" => json!({
            "homeserver_url": "${var.matrix_homeserver_url}",
            "domain": "${var.matrix_domain}",
            "admin_token": "${var.matrix_admin_token}",
            "bridge_command_room_id": "${var.matrix_bridge_command_room_id}",
            "slack_team_id": "${try(local.matrix_slack_team_id, \"\")}",
            "slack_relay_login_id": "${var.matrix_slack_relay_login_id}",
        }),
        "garage" => json!({
            "endpoint": "${var.garage_admin_endpoint}",
            "token": "${var.garage_admin_token}",
        }),
        "random" | "null" | "external" => return None,
        other => panic!("unknown provider config: {other}"),
    })
}

fn provider_vars(name: &str) -> &'static [&'static str] {
    match name {
        "keycloak" => &[
            "keycloak_url",
            "keycloak_realm",
            "keycloak_client_id",
            "keycloak_client_secret",
        ],
        "forgejo" => &["forgejo_url", "forgejo_token"],
        "github" => &["github_org", "github_token"],
        "restapi" => &["forgejo_url", "forgejo_token"],
        "vault" => &[
            "vault_addr",
            "vault_approle_role_id",
            "vault_approle_secret_id",
        ],
        "litellm" => &["litellm_url"],
        "discord" => &["discord_token"],
        "google" => &["google_credentials_json", "google_project_id"],
        "bitwarden" => &[
            "vaultwarden_url",
            "vaultwarden_email",
            "vaultwarden_master_password",
        ],
        "sentry" => &["sentry_token", "sentry_base_url"],
        "posthog" => &["posthog_token", "posthog_host", "posthog_organization_id"],
        "synapse" => &[
            "matrix_homeserver_url",
            "matrix_domain",
            "matrix_admin_token",
            "matrix_bridge_command_room_id",
            "matrix_slack_relay_login_id",
        ],
        "garage" => &["garage_admin_endpoint", "garage_admin_token"],
        "random" | "null" | "external" => &[],
        other => panic!("unknown provider vars: {other}"),
    }
}

fn variable_decl(name: &str) -> Value {
    let string_default = |d: &str| json!({ "type": "string", "default": d });
    let sensitive = json!({ "type": "string", "sensitive": true });

    match name {
        "github_org" => string_default("ScottyLabs"),
        "github_token" => sensitive,
        "forgejo_token" => sensitive,
        "keycloak_url" => string_default("https://idp.scottylabs.org"),
        "keycloak_realm" => string_default("scottylabs"),
        "keycloak_client_id" => sensitive,
        "keycloak_client_secret" => sensitive,
        "discord_token" => sensitive,
        "vaultwarden_url" => string_default("https://vault.scottylabs.org"),
        "vaultwarden_email" => sensitive,
        "vaultwarden_master_password" => sensitive,
        "google_credentials_json" => sensitive,
        "google_project_id" => string_default("sl-governance"),
        "vault_addr" => string_default("https://secrets.scottylabs.org"),
        "vault_approle_role_id" => sensitive,
        "vault_approle_secret_id" => sensitive,
        "kennel_webhook_url" => string_default("https://kennel.scottylabs.org/webhook"),
        "docs_webhook_url" => string_default("https://webhooks.scottylabs.org/hooks/docs-diagrams"),
        "sentry_organization" => string_default("scottylabs"),
        "sentry_token" => sensitive,
        "sentry_base_url" => string_default("https://sentry.io/api/"),
        "posthog_token" => sensitive,
        "posthog_host" => string_default("https://us.posthog.com"),
        "posthog_organization_id" => json!({ "type": "string" }),
        "matrix_homeserver_url" => string_default("https://matrix.doggylabs.org"),
        "matrix_domain" => string_default("doggylabs.org"),
        "matrix_admin_token" => sensitive,
        "matrix_bridge_command_room_id" => json!({
            "type": "string",
            "default": "",
            "description": "Optional Matrix room ID for !discord create-portal when a portal is missing.",
        }),
        "matrix_slack_relay_login_id" => json!({
            "type": "string",
            "default": "",
            "description": "mautrix-slack relay login ID from `list-logins` after `login app` in @slack",
        }),
        "garage_admin_endpoint" => string_default("http://127.0.0.1:3903"),
        "garage_admin_token" => sensitive,
        "garage_s3_endpoint" => string_default("https://s3.scottylabs.org"),
        "cdn_base_url" => string_default("https://cdn.scottylabs.org"),
        "litellm_url" => string_default("https://litellm.scottylabs.org"),
        other => panic!("unknown variable: {other}"),
    }
}

pub fn framework(
    axis: &str,
    providers: &[&str],
    extra_vars: &[&str],
    forgejo_url: &str,
) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    let mut required = Map::new();
    for p in providers {
        required.insert((*p).to_string(), required_provider(p));
    }
    tf.set_terraform(json!({
        "backend": { "s3": backend_s3(axis) },
        "required_providers": Value::Object(required),
    }));

    for p in providers {
        if let Some(cfg) = provider_config(p) {
            tf.add_provider(p, cfg);
        }
    }

    let mut vars = std::collections::BTreeSet::new();
    for p in providers {
        vars.extend(provider_vars(p).iter().copied());
    }
    vars.extend(extra_vars.iter().copied());
    for v in vars {
        let decl = if v == "forgejo_url" {
            json!({ "type": "string", "default": forgejo_url })
        } else {
            variable_decl(v)
        };
        tf.add_variable(v, decl);
    }

    tf
}

pub fn add_shared_data(tf: &mut TfJsonFile, name: &str) {
    match name {
        "keycloak_realm" => tf.add_data("keycloak_realm", "this", json!({ "realm": "scottylabs" })),
        "forgejo_organization" => tf.add_data(
            "forgejo_organization",
            "this",
            json!({ "name": "ScottyLabs" }),
        ),
        "vault_auth_backend_oidc" => {
            tf.add_data("vault_auth_backend", "oidc", json!({ "path": "oidc" }))
        }
        "litellm_master_key" => tf.add_data(
            "vault_kv_secret_v2",
            "litellm_master_key",
            json!({ "mount": "secret", "name": "infra/litellm-master-key" }),
        ),
        other => panic!("unknown shared data source: {other}"),
    }
}

pub fn atlantis_yaml(axes: &[&str]) -> String {
    let mut out =
        String::from("version: 3\nparallel_plan: true\nparallel_apply: true\nprojects:\n");
    for ax in axes {
        out.push_str(&format!(
            "  - name: {ax}\n    dir: tofu/{ax}\n    autoplan:\n      enabled: true\n      when_modified:\n        - \"*.tf\"\n        - \"*.tf.json\"\n    apply_requirements: [mergeable, undiverged]\n"
        ));
    }
    out
}
