use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::org::RepoVisibility;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TeamFile {
    pub team: Team,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct GroupFields {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub public_url: Option<String>,
    #[serde(default)]
    pub leads: Vec<String>,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub repos: Vec<Repo>,
    #[serde(default)]
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub figma_projects: Vec<String>,
}

impl GroupFields {
    pub fn all_members(&self) -> impl Iterator<Item = &str> {
        self.leads
            .iter()
            .chain(self.members.iter())
            .map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Team {
    #[serde(flatten)]
    pub group: GroupFields,
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl Team {
    pub fn groups(&self) -> impl Iterator<Item = &GroupFields> {
        std::iter::once(&self.group).chain(self.projects.iter().map(|p| &p.group))
    }

    // True when the team owns repos directly, not only through projects
    pub fn has_own_group(&self) -> bool {
        self.projects.is_empty() || !self.group.repos.is_empty()
    }

    pub fn repos(&self) -> impl Iterator<Item = &Repo> {
        self.groups().flat_map(|g| g.repos.iter())
    }

    pub fn channels(&self) -> impl Iterator<Item = &Channel> {
        self.groups().flat_map(|g| g.channels.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Project {
    #[serde(flatten)]
    pub group: GroupFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocsType {
    Starlight,
    Rust,
    Openapi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    Kennel,
    Sentry,
    Posthog,
    OidcClient,
    AdminClient,
}

fn default_docs_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Repo {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub visibility: Option<RepoVisibility>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default = "default_docs_enabled")]
    pub docs: bool,
    pub docs_type: Option<DocsType>,
    pub openapi_spec: Option<String>,
    pub export_command: Option<String>,
}

impl Repo {
    pub fn has(&self, feature: Feature) -> bool {
        self.features.contains(&feature)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Channel {
    pub discord: Option<String>,
    pub slack: Option<String>,
}
