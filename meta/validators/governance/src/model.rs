use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Contributor {
    pub name: String,
    pub github: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
    pub repos: Vec<String>,
}

// we do not care about enforcing website vs. websites exclusivity here
#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websites: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
