use std::collections::HashSet;
use std::process::Command;

use governance_schema::team::TeamFile;

use crate::loader::GovernanceData;

pub struct PrCheckResult {
    pub passed: bool,
    pub issues: Vec<String>,
}

pub fn check_pr(
    data: &GovernanceData,
    author: &str,
    base_ref: &str,
    changed_files: &[String],
) -> PrCheckResult {
    let td = &data.org.org.tech_director;
    let is_admin = author == td || is_devops_lead(data, author);

    if is_admin {
        return PrCheckResult {
            passed: true,
            issues: vec![],
        };
    }

    let mut issues = Vec::new();

    for file in changed_files {
        if let Some(slug) = file
            .strip_prefix("data/teams/")
            .and_then(|f| f.strip_suffix(".toml"))
        {
            check_team_file_change(data, author, base_ref, slug, &mut issues);
        } else if file == "data/org.toml" {
            issues.push(format!("{author} cannot modify org.toml"));
        } else if file.starts_with("crates/")
            || file.starts_with("tofu/")
            || file.starts_with("schemas/")
            || file.starts_with(".forgejo/")
        {
            issues.push(format!("{author} cannot modify {file}"));
        }
    }

    PrCheckResult {
        passed: issues.is_empty(),
        issues,
    }
}

fn check_team_file_change(
    data: &GovernanceData,
    author: &str,
    base_ref: &str,
    slug: &str,
    issues: &mut Vec<String>,
) {
    let new_team = data.teams.iter().find(|t| t.team.group.slug == slug);
    let old_content = git_show(base_ref, &format!("data/teams/{slug}.toml"));

    let old_team: Option<TeamFile> = old_content.as_deref().and_then(|c| toml::from_str(c).ok());

    // Team creation
    if old_team.is_none() {
        let is_lead = is_lead_anywhere(data, author);
        if !is_lead {
            issues.push(format!(
                "{author} cannot create team {slug}"
            ));
        }
        return;
    }

    let old_team = old_team.unwrap();
    let new_team = match new_team {
        Some(t) => t,
        None => {
            issues.push(format!(
                "{author} cannot delete team {slug}"
            ));
            return;
        }
    };

    let is_lead_of_team = old_team.team.group.leads.iter().any(|l| l == author);
    if is_lead_of_team {
        return;
    }

    let diff = diff_team(&old_team, new_team);

    for change in &diff {
        match change {
            TeamDiff::MemberAdded(member) if member == author => {}
            TeamDiff::MemberRemoved(member) if member == author => {}
            TeamDiff::ProjectMemberAdded(_, member) if member == author => {}
            TeamDiff::ProjectMemberRemoved(_, member) if member == author => {}
            other => {
                issues.push(format!(
                    "{author} cannot make change in team {slug}: {other}"
                ));
            }
        }
    }
}

fn is_devops_lead(data: &GovernanceData, username: &str) -> bool {
    data.teams
        .iter()
        .filter(|t| t.team.group.slug == "devops")
        .any(|t| t.team.group.leads.iter().any(|l| l == username))
}

fn is_lead_anywhere(data: &GovernanceData, username: &str) -> bool {
    data.teams.iter().any(|t| {
        t.team.group.leads.iter().any(|l| l == username)
            || t.team
                .projects
                .iter()
                .any(|p| p.group.leads.iter().any(|l| l == username))
    })
}

fn git_show(base_ref: &str, path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["show", &format!("{base_ref}:{path}")])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        None
    }
}

#[derive(Debug)]
enum TeamDiff {
    MemberAdded(String),
    MemberRemoved(String),
    ProjectMemberAdded(String, String),
    ProjectMemberRemoved(String, String),
    LeadsChanged,
    ReposChanged,
    ChannelsChanged,
    FigmaChanged,
    ProjectsStructureChanged,
    NameChanged,
    DescriptionChanged,
}

impl std::fmt::Display for TeamDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamDiff::MemberAdded(m) => write!(f, "added member {m}"),
            TeamDiff::MemberRemoved(m) => write!(f, "removed member {m}"),
            TeamDiff::ProjectMemberAdded(p, m) => write!(f, "added member {m} to project {p}"),
            TeamDiff::ProjectMemberRemoved(p, m) => {
                write!(f, "removed member {m} from project {p}")
            }
            TeamDiff::LeadsChanged => write!(f, "modified leads"),
            TeamDiff::ReposChanged => write!(f, "modified repos"),
            TeamDiff::ChannelsChanged => write!(f, "modified channels"),
            TeamDiff::FigmaChanged => write!(f, "modified figma projects"),
            TeamDiff::ProjectsStructureChanged => write!(f, "modified project structure"),
            TeamDiff::NameChanged => write!(f, "modified name"),
            TeamDiff::DescriptionChanged => write!(f, "modified description"),
        }
    }
}

fn diff_team(old: &TeamFile, new: &TeamFile) -> Vec<TeamDiff> {
    let mut diffs = Vec::new();

    if old.team.group.name != new.team.group.name {
        diffs.push(TeamDiff::NameChanged);
    }
    if old.team.group.description != new.team.group.description {
        diffs.push(TeamDiff::DescriptionChanged);
    }
    if old.team.group.leads != new.team.group.leads {
        diffs.push(TeamDiff::LeadsChanged);
    }

    diff_members(
        &old.team.group.members,
        &new.team.group.members,
        |m| TeamDiff::MemberAdded(m),
        |m| TeamDiff::MemberRemoved(m),
        &mut diffs,
    );

    if old.team.group.repos != new.team.group.repos {
        diffs.push(TeamDiff::ReposChanged);
    }
    if old.team.group.channels != new.team.group.channels {
        diffs.push(TeamDiff::ChannelsChanged);
    }
    if old.team.group.figma_projects != new.team.group.figma_projects {
        diffs.push(TeamDiff::FigmaChanged);
    }

    let old_project_slugs: HashSet<&str> = old
        .team
        .projects
        .iter()
        .map(|p| p.group.slug.as_str())
        .collect();
    let new_project_slugs: HashSet<&str> = new
        .team
        .projects
        .iter()
        .map(|p| p.group.slug.as_str())
        .collect();

    if old_project_slugs != new_project_slugs {
        diffs.push(TeamDiff::ProjectsStructureChanged);
    } else {
        for old_project in &old.team.projects {
            let new_project = new
                .team
                .projects
                .iter()
                .find(|p| p.group.slug == old_project.group.slug);
            if let Some(new_project) = new_project {
                let proj_slug = &old_project.group.slug;

                if old_project.group.leads != new_project.group.leads {
                    diffs.push(TeamDiff::LeadsChanged);
                }
                if old_project.group.repos != new_project.group.repos {
                    diffs.push(TeamDiff::ReposChanged);
                }
                if old_project.group.channels != new_project.group.channels {
                    diffs.push(TeamDiff::ChannelsChanged);
                }
                if old_project.group.figma_projects != new_project.group.figma_projects {
                    diffs.push(TeamDiff::FigmaChanged);
                }

                diff_members(
                    &old_project.group.members,
                    &new_project.group.members,
                    |m| TeamDiff::ProjectMemberAdded(proj_slug.clone(), m),
                    |m| TeamDiff::ProjectMemberRemoved(proj_slug.clone(), m),
                    &mut diffs,
                );
            }
        }
    }

    diffs
}

fn diff_members(
    old: &[String],
    new: &[String],
    on_add: impl Fn(String) -> TeamDiff,
    on_remove: impl Fn(String) -> TeamDiff,
    diffs: &mut Vec<TeamDiff>,
) {
    let old_set: HashSet<&str> = old.iter().map(|s| s.as_str()).collect();
    let new_set: HashSet<&str> = new.iter().map(|s| s.as_str()).collect();

    for added in new_set.difference(&old_set) {
        diffs.push(on_add(added.to_string()));
    }
    for removed in old_set.difference(&new_set) {
        diffs.push(on_remove(removed.to_string()));
    }
}
