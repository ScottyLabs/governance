use governance_core::loader::GovernanceData;
use governance_schema::org::ForgeType;
use governance_schema::team::{GroupFields, Repo, TeamFile};
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_repos(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    if org.github.is_none() {
        return tf;
    }
    let forgejo_url = org.forgejo.as_ref().map(|f| f.url());
    let is_default = org.default_forge == ForgeType::Github;

    for team in &data.teams {
        for (repo, is_mirror) in repos_for_github(team, is_default) {
            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));

            if is_mirror {
                let codeberg_url = format!(
                    "{}/{}/{}",
                    forgejo_url.unwrap_or("https://codeberg.org"),
                    org.forgejo.as_ref().map(|f| f.org.as_str()).unwrap_or(""),
                    repo.name
                );
                tf.add_resource(
                    "github_repository",
                    &key,
                    json!({
                        "name": repo.name,
                        "description": "READ-ONLY MIRROR",
                        "homepage_url": codeberg_url,
                        "visibility": "public",
                        "has_issues": false,
                        "has_projects": false,
                        "has_wiki": false,
                        "has_discussions": false,
                        "has_downloads": false,
                    }),
                );

                tf.add_resource(
                    "tls_private_key",
                    &format!("{key}_mirror_key"),
                    json!({
                        "algorithm": "ED25519",
                    }),
                );

                tf.add_resource(
                    "github_repository_deploy_key",
                    &format!("{key}_mirror_deploy_key"),
                    json!({
                        "repository": format!("${{github_repository.{key}.name}}"),
                        "title": "Codeberg Mirroring",
                        "key": format!("${{tls_private_key.{key}_mirror_key.public_key_openssh}}"),
                        "read_only": false,
                    }),
                );

                tf.add_resource(
                    "github_repository_ruleset",
                    &format!("{key}_ruleset"),
                    json!({
                        "name": "Default",
                        "repository": format!("${{github_repository.{key}.name}}"),
                        "target": "branch",
                        "enforcement": "active",
                        "conditions": {
                            "ref_name": {
                                "include": ["~ALL"],
                                "exclude": [],
                            },
                        },
                        "bypass_actors": [
                            {
                                "actor_id": format!("${{github_repository_deploy_key.{key}_mirror_deploy_key.etag}}"),
                                "actor_type": "DeployKey",
                                "bypass_mode": "always",
                            },
                        ],
                        "rules": {
                            "creation": true,
                            "update": true,
                            "deletion": true,
                            "non_fast_forward": true,
                            "pull_request": {
                                "required_approving_review_count": 1,
                            },
                        },
                    }),
                );
            } else {
                let visibility = repo
                    .visibility
                    .as_ref()
                    .unwrap_or(&org.defaults.repo_visibility);
                tf.add_resource(
                    "github_repository",
                    &key,
                    json!({
                        "name": repo.name,
                        "description": repo.description.as_deref().unwrap_or(""),
                        "visibility": format!("{visibility:?}").to_lowercase(),
                        "default_branch": org.defaults.default_branch,
                        "allow_squash_merge": org.defaults.allow_squash_merge,
                        "allow_merge_commit": org.defaults.allow_merge_commit,
                        "allow_rebase_merge": org.defaults.allow_rebase_merge,
                        "has_issues": true,
                        "has_projects": false,
                        "has_wiki": false,
                    }),
                );
            }
        }
    }

    tf
}

pub fn generate_teams(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.github.is_none() {
        return tf;
    }

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let name = team.team.group.name.as_deref().unwrap_or(slug);

        tf.add_resource(
            "github_team",
            slug,
            json!({
                "name": name,
                "privacy": "closed",
            }),
        );
    }

    tf
}

pub fn generate_team_memberships(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.github.is_none() {
        return tf;
    }

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let team_id = format!("${{github_team.{slug}.id}}");

        let all = team.team.group.all_members()
            .chain(team.team.projects.iter().flat_map(|p| p.group.all_members()));

        for username in all {
            let key = username.replace('-', "_");
            let is_lead = team.team.group.leads.iter().any(|l| l == username);

            tf.add_resource(
                "github_team_membership",
                &format!("{slug}_{key}"),
                json!({
                    "team_id": team_id,
                    "username": format!("${{data.external.identity_{key}.result.github}}"),
                    "role": if is_lead { "maintainer" } else { "member" },
                }),
            );
        }
    }

    tf
}

fn repos_for_github<'a>(team: &'a TeamFile, is_default: bool) -> Vec<(&'a Repo, bool)> {
    let mut repos = Vec::new();
    collect_repos(&team.team.group, is_default, &mut repos);
    for project in &team.team.projects {
        collect_repos(&project.group, is_default, &mut repos);
    }
    repos
}

fn collect_repos<'a>(group: &'a GroupFields, is_default: bool, repos: &mut Vec<(&'a Repo, bool)>) {
    for repo in &group.repos {
        match &repo.forge {
            Some(ForgeType::Github) => repos.push((repo, false)),
            Some(ForgeType::Forgejo) => repos.push((repo, true)),
            None if is_default => repos.push((repo, false)),
            None => repos.push((repo, true)),
        }
    }
}
