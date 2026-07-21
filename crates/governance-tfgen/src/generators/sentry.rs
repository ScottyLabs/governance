use governance_core::loader::GovernanceData;
use governance_schema::team::Repo;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        let repos: Vec<&Repo> = team
            .team
            .repos()
            .filter(|r| r.features.sentry.is_some())
            .collect();
        if repos.is_empty() {
            continue;
        }

        let team_slug = &team.team.group.slug;
        let team_name = team.team.group.name.as_str();

        tf.add_resource(
            "sentry_team",
            team_slug,
            json!({
                "organization": "${var.sentry_organization}",
                "name": team_name,
                "slug": team_slug,
            }),
        );

        for repo in repos {
            let key = format!("{}_{}", team_slug, repo.name.replace('-', "_"));

            let mut project = json!({
                "organization": "${var.sentry_organization}",
                "teams": [format!("${{sentry_team.{team_slug}.slug}}")],
                "name": repo.name.clone(),
                "slug": repo.name.clone(),
            });
            match repo
                .features
                .sentry
                .as_ref()
                .and_then(|s| s.platform.as_ref())
            {
                Some(platform) => project["platform"] = json!(platform),
                None => project["lifecycle"] = json!({ "ignore_changes": ["platform"] }),
            }
            tf.add_resource("sentry_project", &key, project);

            tf.add_resource(
                "sentry_key",
                &key,
                json!({
                    "organization": "${var.sentry_organization}",
                    "project": format!("${{sentry_project.{key}.slug}}"),
                    "name": "default",
                }),
            );

            tf.add_resource(
                "vault_kv_secret_v2",
                &format!("{key}_sentry_dsn"),
                json!({
                    "mount": "secret",
                    "name": format!("secretspec/{}/prod/SENTRY_DSN", repo.name),
                    "data_json": format!(
                        "${{jsonencode({{ value = sentry_key.{key}.dsn }})}}"
                    ),
                }),
            );
        }
    }

    tf
}
