use governance_core::loader::GovernanceData;
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

    for team in &data.teams {
        for repo in repos_for_forgejo(team) {
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
        let description = team.team.group.name.as_str();

        tf.add_resource(
            "forgejo_team",
            slug,
            json!({
                "name": slug,
                "description": description,
                "organization_id": "${data.forgejo_organization.this.id}",
                "permission": "write",
                "units_map": {
                    "repo.code": "write",
                    "repo.issues": "write",
                    "repo.pulls": "write",
                },
            }),
        );

        let admin_units = json!({
            "repo.code": "admin",
            "repo.issues": "admin",
            "repo.pulls": "admin",
        });

        if team.team.projects.is_empty() {
            if !team.team.group.leads.is_empty() {
                let key = format!("{slug}_leads");
                tf.add_resource(
                    "forgejo_team",
                    &key,
                    json!({
                        "name": format!("{slug}-leads"),
                        "description": format!("{description} leads"),
                        "organization_id": "${data.forgejo_organization.this.id}",
                        "includes_all_repositories": false,
                        "can_create_org_repo": false,
                        "permission": "admin",
                        "units_map": admin_units,
                    }),
                );
            }
        } else {
            if !team.team.group.repos.is_empty() && !team.team.group.leads.is_empty() {
                let key = format!("{slug}_leads");
                tf.add_resource(
                    "forgejo_team",
                    &key,
                    json!({
                        "name": format!("{slug}-leads"),
                        "description": format!("{description} leads"),
                        "organization_id": "${data.forgejo_organization.this.id}",
                        "includes_all_repositories": false,
                        "can_create_org_repo": false,
                        "permission": "admin",
                        "units_map": admin_units,
                    }),
                );
            }
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                if team.team.group.leads.is_empty() && project.group.leads.is_empty() {
                    continue;
                }
                let key = format!("{project_slug}_leads");
                tf.add_resource(
                    "forgejo_team",
                    &key,
                    json!({
                        "name": format!("{project_slug}-leads"),
                        "description": format!("{} leads", project.group.name),
                        "organization_id": "${data.forgejo_organization.this.id}",
                        "includes_all_repositories": false,
                        "can_create_org_repo": false,
                        "permission": "admin",
                        "units_map": admin_units,
                    }),
                );
            }
        }
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

        if team.team.projects.is_empty() {
            if !team.team.group.leads.is_empty() {
                let leads_key = format!("{slug}_leads");
                let leads_id = format!("${{forgejo_team.{leads_key}.id}}");
                for username in &team.team.group.leads {
                    let key = username.replace('-', "_");
                    tf.add_resource(
                        "forgejo_team_member",
                        &format!("{leads_key}_{key}"),
                        json!({
                            "team_id": leads_id,
                            "user": format!("${{data.external.identity_{key}.result.codeberg}}"),
                        }),
                    );
                }
            }
        } else {
            if !team.team.group.repos.is_empty() && !team.team.group.leads.is_empty() {
                let leads_key = format!("{slug}_leads");
                let leads_id = format!("${{forgejo_team.{leads_key}.id}}");
                for username in &team.team.group.leads {
                    let key = username.replace('-', "_");
                    tf.add_resource(
                        "forgejo_team_member",
                        &format!("{leads_key}_{key}"),
                        json!({
                            "team_id": leads_id,
                            "user": format!("${{data.external.identity_{key}.result.codeberg}}"),
                        }),
                    );
                }
            }
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                if team.team.group.leads.is_empty() && project.group.leads.is_empty() {
                    continue;
                }
                let leads_key = format!("{project_slug}_leads");
                let leads_id = format!("${{forgejo_team.{leads_key}.id}}");
                for username in &team.team.group.leads {
                    let key = username.replace('-', "_");
                    tf.add_resource(
                        "forgejo_team_member",
                        &format!("{leads_key}_{key}"),
                        json!({
                            "team_id": leads_id,
                            "user": format!("${{data.external.identity_{key}.result.codeberg}}"),
                        }),
                    );
                }
                for username in &project.group.leads {
                    if team.team.group.leads.iter().any(|l| l == username) {
                        continue;
                    }
                    let key = username.replace('-', "_");
                    tf.add_resource(
                        "forgejo_team_member",
                        &format!("{leads_key}_{key}"),
                        json!({
                            "team_id": leads_id,
                            "user": format!("${{data.external.identity_{key}.result.codeberg}}"),
                        }),
                    );
                }
            }
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

    for team in &data.teams {
        for repo in repos_for_forgejo(team) {
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

    for team in &data.teams {
        for repo in repos_for_forgejo(team) {
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

pub fn generate_team_repos(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let forgejo = match &org.forgejo {
        Some(f) => f,
        None => return tf,
    };

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let team_ref = format!("${{forgejo_team.{slug}.id}}");

        for repo in repos_for_forgejo(team) {
            let key = format!("{}_{}", slug, repo.name.replace('-', "_"));

            tf.add_resource(
                "forgejo_team_repository",
                &key,
                json!({
                    "team_id": team_ref,
                    "owner": forgejo.org,
                    "repository": repo.name,
                    "depends_on": [
                        format!("forgejo_repository.{key}"),
                        format!("forgejo_team.{slug}"),
                    ],
                }),
            );
        }

        if team.team.projects.is_empty() {
            if !team.team.group.leads.is_empty() {
                let leads_key = format!("{slug}_leads");
                let leads_ref = format!("${{forgejo_team.{leads_key}.id}}");
                for repo in &team.team.group.repos {
                    let repo_key = format!("{}_{}", slug, repo.name.replace('-', "_"));
                    let key = format!("{leads_key}_{}", repo.name.replace('-', "_"));
                    tf.add_resource(
                        "forgejo_team_repository",
                        &key,
                        json!({
                            "team_id": leads_ref,
                            "owner": forgejo.org,
                            "repository": repo.name,
                            "depends_on": [
                                format!("forgejo_repository.{repo_key}"),
                                format!("forgejo_team.{leads_key}"),
                            ],
                        }),
                    );
                }
            }
        } else {
            if !team.team.group.repos.is_empty() && !team.team.group.leads.is_empty() {
                let leads_key = format!("{slug}_leads");
                let leads_ref = format!("${{forgejo_team.{leads_key}.id}}");
                for repo in &team.team.group.repos {
                    let repo_key = format!("{}_{}", slug, repo.name.replace('-', "_"));
                    let key = format!("{leads_key}_{}", repo.name.replace('-', "_"));
                    tf.add_resource(
                        "forgejo_team_repository",
                        &key,
                        json!({
                            "team_id": leads_ref,
                            "owner": forgejo.org,
                            "repository": repo.name,
                            "depends_on": [
                                format!("forgejo_repository.{repo_key}"),
                                format!("forgejo_team.{leads_key}"),
                            ],
                        }),
                    );
                }
            }
            for project in &team.team.projects {
                let project_slug = &project.group.slug;
                if team.team.group.leads.is_empty() && project.group.leads.is_empty() {
                    continue;
                }
                let leads_key = format!("{project_slug}_leads");
                let leads_ref = format!("${{forgejo_team.{leads_key}.id}}");
                for repo in &project.group.repos {
                    let repo_key = format!("{}_{}", slug, repo.name.replace('-', "_"));
                    let key = format!("{leads_key}_{}", repo.name.replace('-', "_"));
                    tf.add_resource(
                        "forgejo_team_repository",
                        &key,
                        json!({
                            "team_id": leads_ref,
                            "owner": forgejo.org,
                            "repository": repo.name,
                            "depends_on": [
                                format!("forgejo_repository.{repo_key}"),
                                format!("forgejo_team.{leads_key}"),
                            ],
                        }),
                    );
                }
            }
        }
    }

    tf
}

fn repos_for_forgejo(team: &TeamFile) -> Vec<&Repo> {
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
