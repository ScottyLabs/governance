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
    pub ai_gateway: Option<AiGatewayFeature>,
    pub docs: Option<DocsFeature>,
}

impl Features {
    // True when the repo provisions secrets into its secretspec namespace
    pub fn writes_secrets(&self) -> bool {
        self.oidc_client.is_some()
            || self.cdn.is_some()
            || self.posthog.is_some()
            || self.sentry.is_some()
            || self.ai_gateway.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KennelFeature {}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SentryFeature {
    /// Sentry platform slug for the project, e.g. "rust" or "python"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PosthogFeature {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CdnFeature {}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OidcClientFeature {
    /// Also provision a Keycloak admin client with realm-management access
    pub admin: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct AiGatewayFeature {
    /// Monthly budget in USD for the prod key
    pub prod_monthly_budget: f64,
    /// Monthly budget in USD for the key shared by staging, preview, and dev
    pub dev_monthly_budget: f64,
    /// Requests per minute for the prod key
    pub prod_rpm_limit: i64,
    /// Requests per minute for the shared non-prod key
    pub dev_rpm_limit: i64,
    /// Tokens per minute for the prod key
    pub prod_tpm_limit: i64,
    /// Tokens per minute for the shared non-prod key
    pub dev_tpm_limit: i64,
    /// Concurrent in-flight requests for the prod key
    pub prod_max_parallel_requests: i64,
    /// Concurrent in-flight requests for the shared non-prod key
    pub dev_max_parallel_requests: i64,
}

impl Default for AiGatewayFeature {
    fn default() -> Self {
        Self {
            prod_monthly_budget: 20.0,
            dev_monthly_budget: 5.0,
            prod_rpm_limit: 1000,
            dev_rpm_limit: 200,
            prod_tpm_limit: 4_000_000,
            dev_tpm_limit: 1_000_000,
            prod_max_parallel_requests: 100,
            dev_max_parallel_requests: 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DocsFeature {}

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
