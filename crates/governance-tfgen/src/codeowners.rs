use governance_core::loader::GovernanceData;

pub fn generate_codeowners(data: &GovernanceData) -> String {
    let td = &data.org.org.tech_director;

    let mut lines = vec![
        "# AUTO-GENERATED - do not edit manually".to_string(),
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

    lines.join("\n")
}
