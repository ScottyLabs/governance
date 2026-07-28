use governance_core::loader::GovernanceData;
use governance_schema::team::GroupFields;

pub fn generate_codeowners(data: &GovernanceData) -> String {
    let td = &data.org.org.tech_director;
    let td_handle = format!("@{td}");
    let global_owners = team_owners(data, "devops", &td_handle);

    let header = [
        "# AUTO-GENERATED".to_string(),
        format!("* {}", global_owners.join(" ")),
        format!("/data/org.toml @{td}"),
    ];

    let teams = data.teams.iter().map(|team| {
        let owners = team_lead_handles(&team.team.group, &td_handle);
        format!(
            "/data/teams/{}.toml {}",
            team.team.group.slug,
            owners.join(" ")
        )
    });

    let trailer = [
        format!("/tofu/ @{td}"),
        format!("/crates/ @{td}"),
        format!("/.forgejo/ @{td}"),
        String::new(),
    ];

    header
        .into_iter()
        .chain(teams)
        .chain(trailer)
        .collect::<Vec<_>>()
        .join("\n")
}

fn team_owners(data: &GovernanceData, slug: &str, td_handle: &str) -> Vec<String> {
    data.teams
        .iter()
        .find(|t| t.team.group.slug == slug)
        .map_or_else(
            || vec![td_handle.to_string()],
            |t| team_lead_handles(&t.team.group, td_handle),
        )
}

fn team_lead_handles(group: &GroupFields, td_handle: &str) -> Vec<String> {
    let mut owners: Vec<String> = group.leads.iter().map(|l| format!("@{l}")).collect();
    if !owners.iter().any(|o| o == td_handle) {
        owners.push(td_handle.to_string());
    }
    owners
}
