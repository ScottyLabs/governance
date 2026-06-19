use governance_core::loader::GovernanceData;
use governance_schema::{org::KeycloakConnection, team::Feature};
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_groups(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.keycloak.is_none() {
        return tf;
    }
    let realm_id = "${data.keycloak_realm.this.id}";
    let projects_id = "${keycloak_group.projects.id}".to_string();

    for team in &data.teams {
        let slug = &team.team.group.slug;

        if team.team.projects.is_empty() {
            let group_key = format!("project_{slug}");
            tf.add_resource(
                "keycloak_group",
                &group_key,
                json!({
                    "realm_id": realm_id,
                    "parent_id": projects_id,
                    "name": slug,
                }),
            );

            tf.add_resource(
                "keycloak_group",
                &format!("project_{slug}_admins"),
                json!({
                    "realm_id": realm_id,
                    "parent_id": format!("${{keycloak_group.{group_key}.id}}"),
                    "name": "admins",
                }),
            );
        } else {
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                let group_key = format!("project_{slug}_{project_slug}");

                tf.add_resource(
                    "keycloak_group",
                    &group_key,
                    json!({
                        "realm_id": realm_id,
                        "parent_id": projects_id,
                        "name": project_slug,
                    }),
                );

                tf.add_resource(
                    "keycloak_group",
                    &format!("{group_key}_admins"),
                    json!({
                        "realm_id": realm_id,
                        "parent_id": format!("${{keycloak_group.{group_key}.id}}"),
                        "name": "admins",
                    }),
                );
            }
        }
    }

    tf
}

pub fn generate_group_memberships(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.keycloak.is_none() {
        return tf;
    }
    let realm_id = "${data.keycloak_realm.this.id}";

    let user_id = |username: &str| {
        let user_key = username.replace('-', "_");
        format!("${{data.external.identity_{user_key}.result.cmu-saml}}")
    };

    let mut emit = |group_key: &str, members: Vec<String>, leads: Vec<String>| {
        let group_id = format!("${{keycloak_group.{group_key}.id}}");
        let admins_id = format!("${{keycloak_group.{group_key}_admins.id}}");

        tf.add_resource(
            "keycloak_group_memberships",
            group_key,
            json!({
                "realm_id": realm_id,
                "group_id": group_id,
                "members": members,
            }),
        );

        tf.add_resource(
            "keycloak_group_memberships",
            &format!("{group_key}_admins"),
            json!({
                "realm_id": realm_id,
                "group_id": admins_id,
                "members": leads,
            }),
        );
    };

    for team in &data.teams {
        let slug = &team.team.group.slug;

        if team.team.projects.is_empty() {
            let members = team.team.group.all_members().map(user_id).collect();
            let leads = team.team.group.leads.iter().map(|s| user_id(s)).collect();
            emit(&format!("project_{slug}"), members, leads);
        } else {
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                let members = team
                    .team
                    .group
                    .all_members()
                    .chain(project.group.all_members())
                    .map(user_id)
                    .collect();
                let leads = team
                    .team
                    .group
                    .leads
                    .iter()
                    .chain(project.group.leads.iter())
                    .map(|s| user_id(s))
                    .collect();
                emit(&format!("project_{slug}_{project_slug}"), members, leads);
            }
        }
    }

    tf
}

pub fn generate_clients(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let Some(keycloak) = &data.org.org.keycloak else {
        return tf;
    };
    let realm_id = "${data.keycloak_realm.this.id}";
    let mut needs_realm_management = false;

    for team in &data.teams {
        for repo in team
            .team
            .repos()
            .filter(|r| r.has(Feature::OidcClient) || r.has(Feature::AdminClient))
        {
            let name = repo.name.as_str();
            let key = name.replace('-', "_");

            if repo.has(Feature::OidcClient) {
                let staging_key = format!("{key}_staging");
                let dev_key = format!("{key}_dev");
                let staging_id = format!("{name}-staging");
                let dev_id = format!("{name}-dev");

                let clients = [
                    (&key, name, &keycloak.redirect_uri),
                    (&staging_key, staging_id.as_str(), &keycloak.redirect_uri),
                    (&dev_key, dev_id.as_str(), &keycloak.dev_redirect_uri),
                ];
                for (client_key, client_id, redirect) in clients {
                    add_client(&mut tf, realm_id, client_key, client_id, redirect);
                }

                // Preview reuses the staging client
                let profiles = [
                    ("prod", &key, &keycloak.redirect_uri),
                    ("staging", &staging_key, &keycloak.redirect_uri),
                    ("preview", &staging_key, &keycloak.redirect_uri),
                    ("dev", &dev_key, &keycloak.dev_redirect_uri),
                ];
                for (profile, client_key, relay) in profiles {
                    add_oidc_secrets(&mut tf, name, &key, profile, client_key, keycloak, relay);
                }
            }

            if repo.has(Feature::AdminClient) {
                needs_realm_management = true;
                let admin_key = format!("{key}_admin");
                add_admin_client(&mut tf, realm_id, &admin_key, &format!("{name}-admin"));
                for profile in ["prod", "staging", "preview", "dev"] {
                    add_admin_secrets(&mut tf, name, &key, profile, &admin_key);
                }
            }
        }
    }

    if needs_realm_management {
        tf.add_data(
            "keycloak_openid_client",
            "realm_management",
            json!({
                "realm_id": realm_id,
                "client_id": "realm-management",
            }),
        );
    }

    tf
}

fn add_client(tf: &mut TfJsonFile, realm_id: &str, key: &str, client_id: &str, redirect_uri: &str) {
    tf.add_resource(
        "keycloak_openid_client",
        key,
        json!({
            "realm_id": realm_id,
            "client_id": client_id,
            "name": client_id,
            "access_type": "CONFIDENTIAL",
            "standard_flow_enabled": true,
            "direct_access_grants_enabled": false,
            "valid_redirect_uris": [redirect_uri],
            "web_origins": ["+"],
        }),
    );

    tf.add_resource(
        "keycloak_openid_group_membership_protocol_mapper",
        key,
        json!({
            "realm_id": realm_id,
            "client_id": format!("${{keycloak_openid_client.{key}.id}}"),
            "name": "groups",
            "claim_name": "groups",
            "full_path": true,
            "add_to_id_token": true,
            "add_to_access_token": true,
            "add_to_userinfo": true,
        }),
    );
}

fn add_admin_client(tf: &mut TfJsonFile, realm_id: &str, key: &str, client_id: &str) {
    tf.add_resource(
        "keycloak_openid_client",
        key,
        json!({
            "realm_id": realm_id,
            "client_id": client_id,
            "name": client_id,
            "access_type": "CONFIDENTIAL",
            "standard_flow_enabled": false,
            "direct_access_grants_enabled": false,
            "service_accounts_enabled": true,
            "valid_redirect_uris": [],
            "web_origins": [],
        }),
    );

    for role in ["view-users", "manage-users", "view-identity-providers"] {
        tf.add_resource(
            "keycloak_openid_client_service_account_role",
            &format!("{key}_{}", role.replace('-', "_")),
            json!({
                "realm_id": realm_id,
                "service_account_user_id": format!(
                    "${{keycloak_openid_client.{key}.service_account_user_id}}"
                ),
                "client_id": "${data.keycloak_openid_client.realm_management.id}",
                "role": role,
            }),
        );
    }
}

fn add_oidc_secrets(
    tf: &mut TfJsonFile,
    repo: &str,
    key: &str,
    profile: &str,
    client_key: &str,
    keycloak: &KeycloakConnection,
    relay_url: &str,
) {
    let secrets = [
        (
            "OIDC_CLIENT_ID",
            format!("keycloak_openid_client.{client_key}.client_id"),
        ),
        (
            "OIDC_CLIENT_SECRET",
            format!("keycloak_openid_client.{client_key}.client_secret"),
        ),
        ("KEYCLOAK_URL", format!("{:?}", keycloak.url)),
        ("KEYCLOAK_REALM", format!("{:?}", keycloak.realm)),
        ("OAUTH_RELAY_URL", format!("{:?}", relay_url)),
    ];

    for (var, value) in secrets {
        add_secret(
            tf,
            &format!("{key}_{profile}_{}", var.to_lowercase()),
            &format!("secretspec/{repo}/{profile}/{var}"),
            &format!("{var} = {value}"),
        );
    }
}

fn add_admin_secrets(tf: &mut TfJsonFile, repo: &str, key: &str, profile: &str, admin_key: &str) {
    add_secret(
        tf,
        &format!("{key}_{profile}_admin_client_id"),
        &format!("secretspec/{repo}/{profile}/KEYCLOAK_ADMIN_CLIENT_ID"),
        &format!("KEYCLOAK_ADMIN_CLIENT_ID = keycloak_openid_client.{admin_key}.client_id"),
    );
    add_secret(
        tf,
        &format!("{key}_{profile}_admin_client_secret"),
        &format!("secretspec/{repo}/{profile}/KEYCLOAK_ADMIN_CLIENT_SECRET"),
        &format!("KEYCLOAK_ADMIN_CLIENT_SECRET = keycloak_openid_client.{admin_key}.client_secret"),
    );
}

fn add_secret(tf: &mut TfJsonFile, key: &str, name: &str, assignment: &str) {
    tf.add_resource(
        "vault_kv_secret_v2",
        key,
        json!({
            "mount": "secret",
            "name": name,
            "data_json": format!("${{jsonencode({{ {assignment} }})}}"),
        }),
    );
}
