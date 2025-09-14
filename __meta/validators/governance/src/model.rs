use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "camelCase"))]
pub struct Contributor {
    pub full_name: String,
    pub github_username: String,
    pub slack_member_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "camelCase"))]
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
    pub repos: Vec<String>,
    pub slack_channel_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websites: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntityKey {
    pub kind: String, // "repo", "team", "contributor"
    pub name: String, // file_stem
}

impl EntityKey {
    pub fn scoped_id(&self) -> String {
        format!("{}:{}", self.kind, self.name)
    }
}

impl PartialEq for EntityKey {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for EntityKey {}

impl Hash for EntityKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for EntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FileValidationMessages {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationStatistics {
    pub contributors_count: usize,
    pub teams_count: usize,
    pub repos_count: usize,
    pub valid_files_count: usize,
    pub invalid_files_count: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub stats: ValidationStatistics,
    pub files: HashMap<String, FileValidationMessages>,
}
