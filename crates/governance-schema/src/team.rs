use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::org::RepoVisibility;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TeamFile {
    pub team: Team,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GroupFields {
    pub slug: String,
    pub name: Option<String>,
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Team {
    #[serde(flatten)]
    pub group: GroupFields,
    #[serde(default)]
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Project {
    #[serde(flatten)]
    pub group: GroupFields,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Repo {
    pub name: String,
    pub description: Option<String>,
    pub forge: Option<String>,
    pub visibility: Option<RepoVisibility>,
    #[serde(default)]
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Channel {
    pub discord: Option<String>,
    pub slack: Option<String>,
}
