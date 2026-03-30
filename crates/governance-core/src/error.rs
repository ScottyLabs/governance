use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("missing org.toml in {0}")]
    MissingOrgFile(PathBuf),

    #[error("no team files found in {0}")]
    NoTeamFiles(PathBuf),

    #[error("validation failed")]
    Validation(Vec<ValidationError>),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("duplicate team slug: {0}")]
    DuplicateTeamSlug(String),

    #[error("duplicate repo name: {0}")]
    DuplicateRepoName(String),

    #[error("team {team}: lead {lead} also listed as member")]
    LeadAlsoMember { team: String, lead: String },

    #[error("forge not configured: {0}")]
    ForgeNotConfigured(String),

    #[error("DISCORD_TOKEN not set")]
    MissingDiscordToken,

    #[error("SLACK_TOKEN not set")]
    MissingSlackToken,

    #[error("discord channel {channel_id} does not exist or is inaccessible")]
    DiscordChannelNotFound { channel_id: String },

    #[error("slack channel {channel_id} does not exist or is inaccessible")]
    SlackChannelNotFound { channel_id: String },

    #[error("discord API error: {0}")]
    DiscordApiError(String),

    #[error("slack API error: {0}")]
    SlackApiError(String),

    #[error("KEYCLOAK_CLIENT_ID and KEYCLOAK_CLIENT_SECRET must be set")]
    MissingKeycloakCredentials,

    #[error("user {user} not found in keycloak (no linked codeberg account)")]
    KeycloakUserNotFound { user: String },

    #[error("user {user} has not linked their {provider} account in keycloak")]
    MissingIdentityLink { user: String, provider: String },

    #[error("keycloak API error: {0}")]
    KeycloakApiError(String),
}
