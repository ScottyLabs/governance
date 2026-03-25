use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OrgFile {
    pub org: OrgConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OrgConfig {
    pub name: String,
    pub tech_director: String,

    pub forges: BTreeMap<String, ForgeInstance>,
    pub communication: Option<CommunicationConfig>,

    pub keycloak: Option<KeycloakConnection>,
    pub google_groups: Option<GoogleGroupsConnection>,
    pub vaultwarden: Option<VaultwardenConnection>,
    pub figma: Option<FigmaConnection>,

    #[serde(default)]
    pub defaults: OrgDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ForgeType {
    Github,
    Forgejo,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ForgeInstance {
    #[serde(rename = "type")]
    pub forge_type: ForgeType,
    pub org: String,
    pub url: Option<String>,
    #[serde(default)]
    pub default: bool,
}

impl ForgeInstance {
    pub fn url(&self) -> &str {
        self.url.as_deref().unwrap_or(match self.forge_type {
            ForgeType::Github => "https://github.com",
            ForgeType::Forgejo => "https://codeberg.org",
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CommunicationConfig {
    pub discord_guild_id: Option<String>,
    pub slack_workspace: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct KeycloakConnection {
    pub url: String,
    pub realm: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GoogleGroupsConnection {
    pub admin: String,
    pub ops: String,
    pub tech: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VaultwardenConnection {
    pub url: String,
    pub org_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FigmaConnection {
    pub org_id: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OrgDefaults {
    pub repo_visibility: RepoVisibility,
    pub default_branch: String,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase_merge: bool,
    pub required_approvals: u32,
}

impl Default for OrgDefaults {
    fn default() -> Self {
        Self {
            repo_visibility: RepoVisibility::Public,
            default_branch: "main".into(),
            allow_squash_merge: true,
            allow_merge_commit: false,
            allow_rebase_merge: true,
            required_approvals: 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RepoVisibility {
    #[default]
    Public,
    Private,
}
