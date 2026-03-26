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
}
