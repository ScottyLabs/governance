use governance_core::loader::GovernanceData;
use governance_schema::org::ForgeType;
use governance_schema::team::{GroupFields, Repo, TeamFile};
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_repos(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let forgejo = match &org.forgejo {
        Some(f) => f,
        None => return tf,
    };
    let is_default = org.default_forge == ForgeType::Forgejo;

    for team in &data.teams {
        for repo in repos_for_forgejo(team, is_default) {
            let resource_name = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
            let is_private = repo
                .visibility
                .as_ref()
                .unwrap_or(&org.defaults.repo_visibility)
                == &governance_schema::org::RepoVisibility::Private;

            tf.add_resource(
                "forgejo_repository",
                &resource_name,
                json!({
                    "name": repo.name,
                    "description": repo.description.as_deref().unwrap_or(""),
                    "owner": forgejo.org,
                    "auto_init": true,
                    "default_branch": org.defaults.default_branch,
                    "private": is_private,
                    "lifecycle": {
                        "ignore_changes": ["clone_addr"],
                    },
                }),
            );
        }
    }

    tf
}

pub fn generate_teams(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.forgejo.is_none() {
        return tf;
    }

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let name = team.team.group.name.as_deref().unwrap_or(slug);

        tf.add_resource(
            "forgejo_team",
            slug,
            json!({
                "name": name,
                "organization_id": "${data.forgejo_organization.this.id}",
                "permission": "write",
                "units_map": {
                    "repo.code": "write",
                    "repo.issues": "write",
                    "repo.pulls": "write",
                },
            }),
        );
    }

    tf
}

pub fn generate_team_memberships(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.forgejo.is_none() {
        return tf;
    }

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let team_id = format!("${{forgejo_team.{slug}.id}}");

        let all = team.team.group.all_members().chain(
            team.team
                .projects
                .iter()
                .flat_map(|p| p.group.all_members()),
        );

        for username in all {
            let key = username.replace('-', "_");
            tf.add_resource(
                "forgejo_team_member",
                &format!("{slug}_{key}"),
                json!({
                    "team_id": team_id,
                    "user": format!("${{data.external.identity_{key}.result.codeberg}}"),
                }),
            );
        }
    }

    tf
}

pub fn generate_push_mirrors(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let forgejo = match &org.forgejo {
        Some(f) => f,
        None => return tf,
    };
    if org.github.is_none() {
        return tf;
    }
    let github_org = org.github.as_ref().unwrap().org.as_str();
    let is_default = org.default_forge == ForgeType::Forgejo;

    for team in &data.teams {
        for repo in repos_for_forgejo(team, is_default) {
            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
            let github_ssh_url = format!("git@github.com:{github_org}/{}.git", repo.name);

            let local_name = format!("{key}_mirror_data");
            tf.add_local(
                &local_name,
                json!({
                    "remote_address": github_ssh_url,
                    "interval": "8h0m0s",
                    "sync_on_commit": true,
                    "use_ssh": true,
                }),
            );

            tf.add_resource(
                "restapi_object",
                &format!("{key}_push_mirror"),
                json!({
                    "path": format!("/api/v1/repos/{}/{}/push_mirrors", forgejo.org, repo.name),
                    "data": format!("${{jsonencode(local.{local_name})}}"),
                    "id_attribute": "remote_name",
                    "depends_on": [
                        format!("forgejo_repository.{key}"),
                    ],
                }),
            );
        }
    }

    tf
}

pub fn generate_kennel_webhooks(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let forgejo = match &org.forgejo {
        Some(f) => f,
        None => return tf,
    };
    let is_default = org.default_forge == ForgeType::Forgejo;

    for team in &data.teams {
        for repo in repos_for_forgejo(team, is_default) {
            if !repo.kennel {
                continue;
            }

            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
            let local_name = format!("{key}_kennel_webhook_data");

            tf.add_local(
                &local_name,
                json!({
                    "type": "forgejo",
                    "active": true,
                    "config": {
                        "url": "${var.kennel_webhook_url}",
                        "content_type": "json",
                        "secret": "${random_password.kennel_webhook_secret.result}",
                    },
                    "events": ["push", "pull_request"],
                }),
            );

            tf.add_resource(
                "restapi_object",
                &format!("{key}_kennel_webhook"),
                json!({
                    "path": format!("/api/v1/repos/{}/{}/hooks", forgejo.org, repo.name),
                    "data": format!("${{jsonencode(local.{local_name})}}"),
                    "id_attribute": "id",
                    "depends_on": [
                        format!("forgejo_repository.{key}"),
                    ],
                }),
            );
        }
    }

    tf
}

fn repos_for_forgejo(team: &TeamFile, is_default: bool) -> Vec<&Repo> {
    let mut repos = Vec::new();
    collect_repos(&team.team.group, is_default, &mut repos);
    for project in &team.team.projects {
        collect_repos(&project.group, is_default, &mut repos);
    }
    repos
}

fn collect_repos<'a>(group: &'a GroupFields, is_default: bool, repos: &mut Vec<&'a Repo>) {
    for repo in &group.repos {
        let on_forgejo = match &repo.forge {
            Some(ForgeType::Forgejo) => true,
            Some(ForgeType::Github) => false,
            None => is_default,
        };
        if on_forgejo {
            repos.push(repo);
        }
    }
}
