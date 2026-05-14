use governance_core::loader::GovernanceData;
use governance_schema::team::{GroupFields, Repo, TeamFile};
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        let repos = sentry_repos(team);
        if repos.is_empty() {
            continue;
        }

        let team_slug = &team.team.group.slug;
        let team_name = team
            .team
            .group
            .name
            .clone()
            .unwrap_or_else(|| team_slug.clone());

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

            tf.add_resource(
                "sentry_project",
                &key,
                json!({
                    "organization": "${var.sentry_organization}",
                    "teams": [format!("${{sentry_team.{team_slug}.slug}}")],
                    "name": repo.name.clone(),
                    "slug": repo.name.clone(),
                }),
            );

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
                        "${{jsonencode({{ SENTRY_DSN = sentry_key.{key}.dsn }})}}"
                    ),
                }),
            );
        }
    }

    tf
}

fn sentry_repos(team: &TeamFile) -> Vec<&Repo> {
    let mut repos = Vec::new();
    collect_sentry(&team.team.group, &mut repos);
    for project in &team.team.projects {
        collect_sentry(&project.group, &mut repos);
    }
    repos
}

fn collect_sentry<'a>(group: &'a GroupFields, repos: &mut Vec<&'a Repo>) {
    for repo in &group.repos {
        if repo.sentry {
            repos.push(repo);
        }
    }
}
