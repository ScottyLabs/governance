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

/// A feature is enabled by its presence, an empty table enables it with defaults
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Features {
    pub kennel: Option<KennelFeature>,
    pub sentry: Option<SentryFeature>,
    pub posthog: Option<PosthogFeature>,
    pub cdn: Option<CdnFeature>,
    pub oidc_client: Option<OidcClientFeature>,
    pub admin_client: Option<AdminClientFeature>,
    pub ai_gateway: Option<AiGatewayFeature>,
    pub docs: Option<DocsFeature>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KennelFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SentryFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PosthogFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CdnFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OidcClientFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdminClientFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AiGatewayFeature {
    /// Monthly budget in USD for the prod key
    #[serde(default = "default_prod_monthly_budget")]
    pub prod_monthly_budget: f64,
    /// Monthly budget in USD for the key shared by staging, preview, and dev
    #[serde(default = "default_dev_monthly_budget")]
    pub dev_monthly_budget: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocsFeature {}

fn default_prod_monthly_budget() -> f64 {
    20.0
}

fn default_dev_monthly_budget() -> f64 {
    5.0
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
    pub features: Features,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Channel {
    pub discord: Option<String>,
    pub slack: Option<String>,
}
