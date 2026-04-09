use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_groups(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let keycloak = match &data.org.org.keycloak {
        Some(k) => k,
        None => return tf,
    };
    tf.add_data(
        "keycloak_realm",
        "this",
        json!({
            "realm": keycloak.realm,
        }),
    );
    let realm_id = "${data.keycloak_realm.this.id}";

    tf.add_resource(
        "keycloak_group",
        "projects",
        json!({
            "realm_id": realm_id,
            "name": "projects",
        }),
    );

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

    for team in &data.teams {
        let slug = &team.team.group.slug;

        if team.team.projects.is_empty() {
            let group_key = format!("project_{slug}");
            let group_id = format!("${{keycloak_group.{group_key}.id}}");
            let admins_id = format!("${{keycloak_group.{group_key}_admins.id}}");

            for username in team.team.group.all_members() {
                let user_key = username.replace('-', "_");
                let user_id = format!("${{data.external.identity_{user_key}.result.cmu-saml}}");

                tf.add_resource(
                    "keycloak_group_memberships",
                    &format!("{group_key}_{user_key}"),
                    json!({
                        "realm_id": realm_id,
                        "group_id": group_id,
                        "members": [user_id],
                    }),
                );
            }

            for lead in &team.team.group.leads {
                let user_key = lead.replace('-', "_");
                let user_id = format!("${{data.external.identity_{user_key}.result.cmu-saml}}");

                tf.add_resource(
                    "keycloak_group_memberships",
                    &format!("{group_key}_admins_{user_key}"),
                    json!({
                        "realm_id": realm_id,
                        "group_id": admins_id,
                        "members": [user_id],
                    }),
                );
            }
        } else {
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                let group_key = format!("project_{slug}_{project_slug}");
                let group_id = format!("${{keycloak_group.{group_key}.id}}");
                let admins_id = format!("${{keycloak_group.{group_key}_admins.id}}");

                let members = team.team.group.all_members()
                    .chain(project.group.all_members());

                for username in members {
                    let user_key = username.replace('-', "_");
                    let user_id = format!("${{data.external.identity_{user_key}.result.cmu-saml}}");

                    tf.add_resource(
                        "keycloak_group_memberships",
                        &format!("{group_key}_{user_key}"),
                        json!({
                            "realm_id": realm_id,
                            "group_id": group_id,
                            "members": [user_id],
                        }),
                    );
                }

                let leads = team.team.group.leads.iter()
                    .chain(project.group.leads.iter());

                for lead in leads {
                    let user_key = lead.replace('-', "_");
                    let user_id = format!("${{data.external.identity_{user_key}.result.cmu-saml}}");

                    tf.add_resource(
                        "keycloak_group_memberships",
                        &format!("{group_key}_admins_{user_key}"),
                        json!({
                            "realm_id": realm_id,
                            "group_id": admins_id,
                            "members": [user_id],
                        }),
                    );
                }
            }
        }
    }

    tf
}
