use std::path::{Path, PathBuf};

use governance_schema::org::OrgFile;
use governance_schema::team::{GroupFields, TeamFile};

use crate::error::GovernanceError;

pub struct GovernanceData {
    pub org: OrgFile,
    pub teams: Vec<TeamFile>,
}

impl GovernanceData {
    pub fn load(data_dir: &Path) -> Result<Self, GovernanceError> {
        let mut org = load_org(data_dir)?;
        let mut teams = load_teams(data_dir)?;
        normalize_usernames(&mut org, &mut teams);
        Ok(Self { org, teams })
    }

    pub fn all_members(&self) -> Vec<&str> {
        let mut members: Vec<&str> = self
            .teams
            .iter()
            .flat_map(|t| t.team.groups())
            .flat_map(GroupFields::all_members)
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }

    pub fn all_leads(&self) -> Vec<&str> {
        let mut leads: Vec<&str> = self
            .teams
            .iter()
            .flat_map(|t| t.team.groups())
            .flat_map(|g| g.leads.iter().map(String::as_str))
            .collect();
        leads.sort_unstable();
        leads.dedup();
        leads
    }
}

fn load_org(data_dir: &Path) -> Result<OrgFile, GovernanceError> {
    let path = data_dir.join("org.toml");
    if !path.exists() {
        return Err(GovernanceError::MissingOrgFile(data_dir.to_path_buf()));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| GovernanceError::ReadFile {
        path: path.clone(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| GovernanceError::ParseToml { path, source: e })
}

fn load_teams(data_dir: &Path) -> Result<Vec<TeamFile>, GovernanceError> {
    let teams_dir = data_dir.join("teams");
    if !teams_dir.exists() {
        return Err(GovernanceError::NoTeamFiles(teams_dir));
    }

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&teams_dir)
        .map_err(|e| GovernanceError::ReadFile {
            path: teams_dir.clone(),
            source: e,
        })?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    entries.sort();

    let teams = entries
        .iter()
        .map(|path| -> Result<TeamFile, GovernanceError> {
            let content = std::fs::read_to_string(path).map_err(|e| GovernanceError::ReadFile {
                path: path.clone(),
                source: e,
            })?;
            toml::from_str(&content).map_err(|e| GovernanceError::ParseToml {
                path: path.clone(),
                source: e,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if teams.is_empty() {
        return Err(GovernanceError::NoTeamFiles(teams_dir));
    }

    Ok(teams)
}

// Codeberg usernames are case-insensitive so lowercase them to keep terraform resource keys stable
fn normalize_usernames(org: &mut OrgFile, teams: &mut [TeamFile]) {
    org.org.tech_director = org.org.tech_director.to_lowercase();
    for team in teams {
        normalize_group(&mut team.team.group);
        for project in &mut team.team.projects {
            normalize_group(&mut project.group);
        }
    }
}

fn normalize_group(group: &mut GroupFields) {
    for user in group.leads.iter_mut().chain(group.members.iter_mut()) {
        *user = user.to_lowercase();
    }
}
