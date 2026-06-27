use governance_core::loader::GovernanceData;
use governance_schema::team::Feature;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        for repo in team.team.repos().filter(|r| r.has(Feature::Posthog)) {
            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));

            tf.add_resource(
                "posthog_project",
                &key,
                json!({
                    "name": repo.name,
                }),
            );

            tf.add_resource(
                "vault_kv_secret_v2",
                &format!("{key}_posthog_key"),
                json!({
                    "mount": "secret",
                    "name": format!("secretspec/{}/prod/POSTHOG_KEY", repo.name),
                    "data_json": format!(
                        "${{jsonencode({{ value = posthog_project.{key}.api_token }})}}"
                    ),
                }),
            );

            tf.add_resource(
                "vault_kv_secret_v2",
                &format!("{key}_posthog_host"),
                json!({
                    "mount": "secret",
                    "name": format!("secretspec/{}/prod/POSTHOG_HOST", repo.name),
                    "data_json": "${jsonencode({ value = var.posthog_host })}",
                }),
            );
        }
    }

    tf
}
