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
            .map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Team {
    #[serde(flatten)]
    pub group: GroupFields,
    #[serde(default)]
    pub projects: Vec<Project>,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Repo {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<RepoVisibility>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub kennel: bool,
    #[serde(default)]
    pub sentry: bool,
    #[serde(default)]
    pub docs: bool,
    pub docs_type: Option<DocsType>,
    pub docs_dir: Option<String>,
    pub openapi_spec: Option<String>,
    pub export_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Channel {
    pub discord: Option<String>,
    pub slack: Option<String>,
}
