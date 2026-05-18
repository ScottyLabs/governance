use governance_core::loader::GovernanceData;
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

    for team in &data.teams {
        for repo in repos_for_github(team) {
            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
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
                }),
            );

            tf.add_resource(
                "github_repository_deploy_key",
                &format!("{key}_mirror_deploy_key"),
                json!({
                    "repository": format!("${{github_repository.{key}.name}}"),
                    "title": "Codeberg Mirroring",
                    "key": format!("${{restapi_object.{key}_push_mirror.api_data.public_key}}"),
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
                            "actor_id": format!("${{tonumber(element(split(\":\", github_repository_deploy_key.{key}_mirror_deploy_key.id), 1))}}"),
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
        let name = team.team.group.name.as_str();

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

        let all = team.team.group.all_members().chain(
            team.team
                .projects
                .iter()
                .flat_map(|p| p.group.all_members()),
        );

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

fn repos_for_github(team: &TeamFile) -> Vec<&Repo> {
    let mut repos = Vec::new();
    collect_repos(&team.team.group, &mut repos);
    for project in &team.team.projects {
        collect_repos(&project.group, &mut repos);
    }
    repos
}

fn collect_repos<'a>(group: &'a GroupFields, repos: &mut Vec<&'a Repo>) {
    repos.extend(group.repos.iter());
}
