use governance_core::loader::GovernanceData;
use governance_schema::team::{GroupFields, Repo, TeamFile};

pub fn generate_codeowners(data: &GovernanceData) -> String {
    let td = &data.org.org.tech_director;

    let mut lines = vec![
        "# AUTO-GENERATED".to_string(),
        format!("* @{td}"),
        format!("/data/org.toml @{td}"),
    ];

    for team in &data.teams {
        let mut owners: Vec<String> = team
            .team
            .group
            .leads
            .iter()
            .map(|l| format!("@{l}"))
            .collect();
        if !owners.iter().any(|o| o == &format!("@{td}")) {
            owners.push(format!("@{td}"));
        }
        lines.push(format!(
            "/data/teams/{}.toml {}",
            team.team.group.slug,
            owners.join(" ")
        ));
    }

    lines.push(format!("/tofu/ @{td}"));
    lines.push(format!("/crates/ @{td}"));
    lines.push(format!("/.forgejo/ @{td}"));

    lines.push(String::new());
    lines.join("\n")
}

pub fn generate_observability_codeowners(data: &GovernanceData) -> String {
    let td = &data.org.org.tech_director;
    let td_handle = format!("@{td}");

    let mut lines = vec![
        "# AUTO-GENERATED".to_string(),
        format!("* {td_handle}"),
    ];

    let devops_owners = team_owners(data, "devops", &td_handle);
    lines.push(format!(
        "/dashboards/infra/ {}",
        devops_owners.join(" ")
    ));
    lines.push(format!("/alerts/infra/ {}", devops_owners.join(" ")));

    for team in &data.teams {
        let owners = team_lead_handles(&team.team.group, &td_handle);
        for repo in sentry_repos(team) {
            lines.push(format!(
                "/dashboards/{}/ {}",
                repo.name,
                owners.join(" ")
            ));
            lines.push(format!("/alerts/{}/ {}", repo.name, owners.join(" ")));
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

fn team_owners(data: &GovernanceData, slug: &str, td_handle: &str) -> Vec<String> {
    data.teams
        .iter()
        .find(|t| t.team.group.slug == slug)
        .map(|t| team_lead_handles(&t.team.group, td_handle))
        .unwrap_or_else(|| vec![td_handle.to_string()])
}

fn team_lead_handles(group: &GroupFields, td_handle: &str) -> Vec<String> {
    let mut owners: Vec<String> = group
        .leads
        .iter()
        .map(|l| format!("@{l}"))
        .collect();
    if !owners.iter().any(|o| o == td_handle) {
        owners.push(td_handle.to_string());
    }
    owners
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
