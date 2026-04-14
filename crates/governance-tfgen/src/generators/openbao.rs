use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

fn secretspec_policy(project: &str, profiles: &[&str]) -> String {
    profiles
        .iter()
        .flat_map(|profile| {
            [
                format!(
                    "path \"secret/data/secretspec/{project}/{profile}/*\" {{\n  capabilities = [\"create\", \"read\", \"update\", \"delete\", \"list\"]\n}}"
                ),
                format!(
                    "path \"secret/metadata/secretspec/{project}/{profile}/*\" {{\n  capabilities = [\"list\", \"read\", \"delete\"]\n}}"
                ),
            ]
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn generate_project_policies(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        for project in &team.team.projects {
            let slug = &project.group.slug;

            // Dev policy: project members can manage secrets for dev and preview profiles
            tf.add_resource(
                "vault_policy",
                &format!("{slug}_dev"),
                json!({
                    "name": format!("{slug}-dev"),
                    "policy": secretspec_policy(slug, &["dev", "preview"]),
                }),
            );

            // Prod policy: project admins can manage secrets for prod and staging profiles
            tf.add_resource(
                "vault_policy",
                &format!("{slug}_prod"),
                json!({
                    "name": format!("{slug}-prod"),
                    "policy": secretspec_policy(slug, &["prod", "staging"]),
                }),
            );

            // Identity group for project members, linked to dev policy
            tf.add_resource(
                "vault_identity_group",
                &format!("{slug}_members"),
                json!({
                    "name": format!("{slug}-members"),
                    "type": "external",
                    "policies": [format!("{slug}-dev")],
                }),
            );

            tf.add_resource(
                "vault_identity_group_alias",
                &format!("{slug}_members"),
                json!({
                    "name": format!("/projects/{slug}"),
                    "mount_accessor": "${data.vault_auth_backend.oidc.accessor}",
                    "canonical_id": format!("${{vault_identity_group.{slug}_members.id}}"),
                }),
            );

            // Identity group for project admins, linked to prod policy
            tf.add_resource(
                "vault_identity_group",
                &format!("{slug}_admins"),
                json!({
                    "name": format!("{slug}-admins"),
                    "type": "external",
                    "policies": [format!("{slug}-prod")],
                }),
            );

            tf.add_resource(
                "vault_identity_group_alias",
                &format!("{slug}_admins"),
                json!({
                    "name": format!("/projects/{slug}/admins"),
                    "mount_accessor": "${data.vault_auth_backend.oidc.accessor}",
                    "canonical_id": format!("${{vault_identity_group.{slug}_admins.id}}"),
                }),
            );
        }
    }

    tf
}
