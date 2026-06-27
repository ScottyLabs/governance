use governance_core::loader::GovernanceData;
use governance_schema::team::GroupFields;
use serde::Serialize;

#[derive(Serialize)]
struct ProjectCard {
    slug: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    url: String,
    leads: Vec<String>,
    members: Vec<String>,
    repos: Vec<String>,
}

pub fn generate_projects(data: &GovernanceData) -> String {
    let mut cards = Vec::new();

    for team in &data.teams {
        let t = &team.team;

        // Team card aggregates the team and any project without its own card
        if let Some(url) = t.group.public_url.as_deref() {
            let groups: Vec<&GroupFields> = std::iter::once(&t.group)
                .chain(
                    t.projects
                        .iter()
                        .map(|p| &p.group)
                        .filter(|g| g.public_url.is_none()),
                )
                .collect();
            cards.push(card(&t.group, url, &groups, &groups));
        }

        for project in &t.projects {
            if let Some(url) = project.group.public_url.as_deref() {
                let roster = [&t.group, &project.group];
                cards.push(card(&project.group, url, &roster, &[&project.group]));
            }
        }
    }

    let mut out = serde_json::to_string_pretty(&cards).expect("serialize projects");
    out.push('\n');
    out
}

// Leads and members span roster, repos come only from repo_groups
fn card(
    group: &GroupFields,
    url: &str,
    roster: &[&GroupFields],
    repo_groups: &[&GroupFields],
) -> ProjectCard {
    ProjectCard {
        slug: group.slug.clone(),
        name: group.name.clone(),
        description: group.description.clone(),
        url: url.to_string(),
        leads: dedup(roster.iter().flat_map(|g| g.leads.iter())),
        members: dedup(roster.iter().flat_map(|g| g.members.iter())),
        repos: repo_groups
            .iter()
            .flat_map(|g| g.repos.iter())
            .map(|r| r.name.clone())
            .collect(),
    }
}

fn dedup<'a>(items: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        if !out.iter().any(|x| x == item) {
            out.push(item.clone());
        }
    }
    out
}
