use std::path::{Path, PathBuf};

use governance_schema::org::OrgFile;
use governance_schema::team::TeamFile;

use crate::error::GovernanceError;

pub struct GovernanceData {
    pub org: OrgFile,
    pub teams: Vec<TeamFile>,
}

impl GovernanceData {
    pub fn load(data_dir: &Path) -> Result<Self, GovernanceError> {
        let org = load_org(data_dir)?;
        let teams = load_teams(data_dir)?;
        Ok(Self { org, teams })
    }

    pub fn all_members(&self) -> Vec<&str> {
        let mut members: Vec<&str> = self
            .teams
            .iter()
            .flat_map(|t| {
                let team = &t.team.group;
                let project_people = t.team.projects.iter().flat_map(|p| {
                    p.group.leads.iter().chain(p.group.members.iter()).map(|s| s.as_str())
                });
                team.leads.iter()
                    .chain(team.members.iter())
                    .map(|s| s.as_str())
                    .chain(project_people)
            })
            .collect();
        members.sort();
        members.dedup();
        members
    }

    pub fn all_leads(&self) -> Vec<&str> {
        let mut leads: Vec<&str> = self
            .teams
            .iter()
            .flat_map(|t| {
                let project_leads = t.team.projects.iter().flat_map(|p| {
                    p.group.leads.iter().map(|s| s.as_str())
                });
                t.team.group.leads.iter().map(|s| s.as_str()).chain(project_leads)
            })
            .collect();
        leads.sort();
        leads.dedup();
        leads
    }

    pub fn default_forge_name(&self) -> Option<&str> {
        self.org
            .org
            .forges
            .iter()
            .find(|(_, f)| f.default)
            .map(|(name, _)| name.as_str())
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

    let mut teams = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&teams_dir)
        .map_err(|e| GovernanceError::ReadFile {
            path: teams_dir.clone(),
            source: e,
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    entries.sort();

    for path in entries {
        let content =
            std::fs::read_to_string(&path).map_err(|e| GovernanceError::ReadFile {
                path: path.clone(),
                source: e,
            })?;
        let team: TeamFile =
            toml::from_str(&content).map_err(|e| GovernanceError::ParseToml {
                path: path.clone(),
                source: e,
            })?;
        teams.push(team);
    }

    if teams.is_empty() {
        return Err(GovernanceError::NoTeamFiles(teams_dir));
    }

    Ok(teams)
}
