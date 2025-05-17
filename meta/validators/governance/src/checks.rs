use std::collections::HashMap;

use anyhow::{Result, anyhow};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use governance::model::{Contributor, EntityKey, Repo, Team, ValidationError, ValidationWarning};
use log::info;
use reqwest::{Client, StatusCode};

pub fn validate_file_names(
    contributors: &HashMap<EntityKey, Contributor>,
    teams: &HashMap<EntityKey, Team>,
    repos: &HashMap<EntityKey, Repo>,
) -> Vec<ValidationError> {
    info!("Validating file names...");
    let mut errors = Vec::new();

    // Validate contributor filenames match GitHub usernames
    for (key, contributor) in contributors {
        if key.name != contributor.github_username {
            errors.push(ValidationError {
                file: format!("contributors/{}.toml", key),
                message: format!(
                    "Contributor file name '{}' doesn't match GitHub username '{}'",
                    key.name.red().bold(),
                    contributor.github_username.red().bold()
                ),
            });
        }
    }

    // Validate team filenames match team names
    for (key, team) in teams {
        if key.name != team.name {
            errors.push(ValidationError {
                file: format!("teams/{}.toml", key),
                message: format!(
                    "Team file name '{}' doesn't match team name '{}'",
                    key.name.red().bold(),
                    team.name.red().bold()
                ),
            });
        }
    }

    // Validate repo filenames match repo names
    for (key, repo) in repos {
        if key.name != repo.name {
            errors.push(ValidationError {
                file: format!("repos/{}.toml", key),
                message: format!(
                    "Repo file name '{}' doesn't match repo name '{}'",
                    key.name.red().bold(),
                    repo.name.red().bold()
                ),
            });
        }
    }

    errors
}

pub fn validate_cross_references(
    contributors: &HashMap<EntityKey, Contributor>,
    teams: &HashMap<EntityKey, Team>,
    repos: &HashMap<EntityKey, Repo>,
) -> Vec<ValidationError> {
    info!("Validating cross-references...");
    let mut errors = Vec::new();

    // Check that all team members exist in contributors
    for (team_key, team) in teams {
        for member in &team.members {
            let key = EntityKey {
                kind: "contributor".to_string(),
                name: member.clone(),
            };

            if !contributors.contains_key(&key) {
                errors.push(ValidationError {
                    file: format!("teams/{}.toml", team_key),
                    message: format!(
                        "Team '{}' references non-existent contributor: {}",
                        team_key.name.red().bold(),
                        member.red().bold()
                    ),
                });
            }
        }
    }

    // Check that all team repos exist in repos
    for (team_key, team) in teams {
        for repo in &team.repos {
            let key = EntityKey {
                kind: "repo".to_string(),
                name: repo.clone(),
            };

            if !repos.contains_key(&key) {
                errors.push(ValidationError {
                    file: format!("teams/{}.toml", team_key),
                    message: format!(
                        "Team '{}' references non-existent repo: {}",
                        team_key.name.red().bold(),
                        repo.red().bold()
                    ),
                });
            }
        }
    }

    errors
}

async fn check_github_user_exists(github_username: &str, client: &Client) -> Result<bool> {
    let token = std::env::var("GITHUB_TOKEN").unwrap_or_default();
    let mut request = client
        .get(format!("https://api.github.com/users/{}", github_username))
        .header("User-Agent", "ScottyLabs-Governance-Validator");

    if !token.is_empty() {
        request = request.bearer_auth(token);
    }

    let response = request.send().await?;
    let status = response.status();

    match status {
        StatusCode::OK => Ok(true),
        StatusCode::NOT_FOUND => Ok(false),
        StatusCode::FORBIDDEN => Err(anyhow!("Rate limit exceeded or access forbidden",)),
        _ => Err(anyhow!("Unexpected status {}", status,)),
    }
}

pub async fn validate_github_users(
    contributors: &HashMap<EntityKey, Contributor>,
    client: &Client,
) -> (Vec<ValidationError>, Vec<ValidationWarning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut futures = FuturesUnordered::new();

    for (contributor_id, contributor) in contributors {
        futures.push(async move {
            let result = check_github_user_exists(&contributor.github_username, client).await;
            (contributor_id, &contributor.github_username, result)
        });
    }

    while let Some((contributor_id, github, result)) = futures.next().await {
        match result {
            Ok(true) => {}
            Ok(false) => errors.push(ValidationError {
                file: format!("contributors/{}.toml", contributor_id),
                message: format!("GitHub user does not exist: {}", github.red().bold()),
            }),
            Err(e) => warnings.push(ValidationWarning {
                file: format!("contributors/{}.toml", contributor_id),
                message: format!(
                    "Failed to check GitHub user {}: {}",
                    github.yellow().bold(),
                    e
                ),
            }),
        }
    }

    (errors, warnings)
}
