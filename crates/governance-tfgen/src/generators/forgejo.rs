use governance_core::loader::GovernanceData;
use governance_schema::team::{Feature, GroupFields, TeamFile};
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_repos(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let Some(forgejo) = &org.forgejo else {
        return tf;
    };

    for team in &data.teams {
        for repo in team.team.repos() {
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
    }

    for scope in data.teams.iter().flat_map(lead_scopes) {
        tf.add_resource(
            "forgejo_team",
            &scope.key,
            json!({
                "name": scope.name,
                "description": scope.description,
                "organization_id": "${data.forgejo_organization.this.id}",
                "includes_all_repositories": false,
                "can_create_org_repo": false,
                "permission": "admin",
                "units_map": {
                    "repo.code": "admin",
                    "repo.issues": "admin",
                    "repo.pulls": "admin",
                    "repo.releases": "admin",
                    "repo.wiki": "admin",
                    "repo.ext_wiki": "admin",
                    "repo.ext_issues": "admin",
                    "repo.projects": "admin",
                    "repo.packages": "admin",
                    "repo.actions": "admin",
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

    for scope in data.teams.iter().flat_map(lead_scopes) {
        let leads_id = format!("${{forgejo_team.{}.id}}", scope.key);
        for username in scope.leads() {
            let key = username.replace('-', "_");
            tf.add_resource(
                "forgejo_team_member",
                &format!("{}_{key}", scope.key),
                json!({
                    "team_id": leads_id,
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
    let Some(forgejo) = &org.forgejo else {
        return tf;
    };
    let Some(github) = &org.github else {
        return tf;
    };
    let github_org = github.org.as_str();

    for team in &data.teams {
        for repo in team.team.repos() {
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
    let Some(forgejo) = &org.forgejo else {
        return tf;
    };

    for team in &data.teams {
        for repo in team.team.repos() {
            if !repo.has(Feature::Kennel) {
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

pub fn generate_docs_webhooks(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let org = &data.org.org;
    let Some(forgejo) = &org.forgejo else {
        return tf;
    };

    for team in &data.teams {
        for repo in team.team.repos() {
            if !repo.docs || repo.name == "documentation" {
                continue;
            }

            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
            let local_name = format!("{key}_docs_webhook_data");

            tf.add_local(
                &local_name,
                json!({
                    "type": "forgejo",
                    "active": true,
                    "config": {
                        "url": "${var.docs_webhook_url}",
                        "content_type": "json",
                    },
                    "events": ["push"],
                }),
            );

            tf.add_resource(
                "restapi_object",
                &format!("{key}_docs_webhook"),
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
    let Some(forgejo) = &org.forgejo else {
        return tf;
    };

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let team_ref = format!("${{forgejo_team.{slug}.id}}");

        for repo in team.team.repos() {
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
    }

    for scope in data.teams.iter().flat_map(lead_scopes) {
        let leads_ref = format!("${{forgejo_team.{}.id}}", scope.key);
        for repo in scope.group.repos.iter() {
            let repo_key = format!("{}_{}", scope.team_slug, repo.name.replace('-', "_"));
            tf.add_resource(
                "forgejo_team_repository",
                &format!("{}_{}", scope.key, repo.name.replace('-', "_")),
                json!({
                    "team_id": leads_ref,
                    "owner": forgejo.org,
                    "repository": repo.name,
                    "depends_on": [
                        format!("forgejo_repository.{repo_key}"),
                        format!("forgejo_team.{}", scope.key),
                    ],
                }),
            );
        }
    }

    tf
}

struct LeadScope<'a> {
    key: String,
    name: String,
    description: String,
    team_slug: &'a str,
    group: &'a GroupFields,
    extra_leads: &'a [String],
}

impl<'a> LeadScope<'a> {
    // team leads (extra_leads) plus this group's leads, deduped case-insensitively
    fn leads(&self) -> impl Iterator<Item = &str> {
        self.extra_leads.iter().map(String::as_str).chain(
            self.group
                .leads
                .iter()
                .map(String::as_str)
                .filter(move |pl| {
                    !self
                        .extra_leads
                        .iter()
                        .any(|tl| tl.eq_ignore_ascii_case(pl))
                }),
        )
    }
}

fn lead_scopes(team: &TeamFile) -> impl Iterator<Item = LeadScope<'_>> {
    let t = &team.team;
    let slug = t.group.slug.as_str();
    std::iter::once(LeadScope {
        key: format!("{slug}_leads"),
        name: format!("{slug}-leads"),
        description: format!("{} leads", t.group.name),
        team_slug: slug,
        group: &t.group,
        extra_leads: &[],
    })
    .chain(t.projects.iter().map(move |p| LeadScope {
        key: format!("{}_leads", p.group.slug),
        name: format!("{}-leads", p.group.slug),
        description: format!("{} leads", p.group.name),
        team_slug: slug,
        group: &p.group,
        extra_leads: &t.group.leads,
    }))
    .filter(|s| s.leads().next().is_some() && !s.group.repos.is_empty())
}
