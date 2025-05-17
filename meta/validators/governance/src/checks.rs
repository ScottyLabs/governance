use std::collections::HashMap;

use anyhow::{Result, anyhow};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use governance::model::{Contributor, EntityKey, Repo, Team, ValidationError, ValidationWarning};
use log::info;
use reqwest::{Client, StatusCode};
use serde_json::Value;

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

async fn check_slack_id_exists(slack_id: &str, client: &Client) -> Result<bool> {
    let token = std::env::var("SLACK_TOKEN").unwrap_or_default();

    // Slack API always requires authentication
    if token.is_empty() {
        return Err(anyhow!("SLACK_TOKEN environment variable not set"));
    }

    // Determine endpoint and parameter based on ID prefix
    let (endpoint, param_name) = if slack_id.starts_with('U') {
        ("https://slack.com/api/users.info", "user")
    } else if slack_id.starts_with('C') || slack_id.starts_with('G') {
        ("https://slack.com/api/conversations.info", "channel")
    } else {
        return Err(anyhow!("Invalid Slack ID format: {}", slack_id));
    };

    let request = client
        .get(endpoint)
        .query(&[(param_name, slack_id)])
        .header("User-Agent", "ScottyLabs-Governance-Validator")
        .bearer_auth(token);

    let response = request.send().await?;

    // Unlike GitHub API, Slack API always returns HTTP 200 OK
    // The actual success/failure is in the JSON response
    let json: Value = response.json().await?;

    if let Some(ok) = json.get("ok").and_then(|v| v.as_bool()) {
        if ok {
            return Ok(true);
        } else if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
            match error {
                "user_not_found" | "channel_not_found" => return Ok(false),
                "ratelimited" => return Err(anyhow!("Rate limit exceeded")),
                "invalid_auth" => return Err(anyhow!("Invalid authentication")),
                _ => return Err(anyhow!("Slack API error: {}", error)),
            }
        }
    }

    Err(anyhow!("Unexpected response from Slack API"))
}

pub async fn validate_slack_ids(
    contributors: &HashMap<EntityKey, Contributor>,
    teams: &HashMap<EntityKey, Team>,
    client: &Client,
) -> (Vec<ValidationError>, Vec<ValidationWarning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut futures = FuturesUnordered::new();

    for (contributor_id, contributor) in contributors {
        futures.push(async move {
            let result = check_slack_id_exists(&contributor.slack_member_id, client).await;
            (contributor_id, &contributor.slack_member_id, result)
        });
    }

    while let Some((contributor_id, slack_id, result)) = futures.next().await {
        match result {
            Ok(true) => {}
            Ok(false) => errors.push(ValidationError {
                file: format!("contributors/{}.toml", contributor_id),
                message: format!("Slack member ID does not exist: {}", slack_id.red().bold()),
            }),
            Err(e) => warnings.push(ValidationWarning {
                file: format!("contributors/{}.toml", contributor_id),
                message: format!(
                    "Failed to check Slack member ID {}: {}",
                    slack_id.yellow().bold(),
                    e
                ),
            }),
        }
    }

    // Reset futures for channel validations
    let mut futures = FuturesUnordered::new();

    for (team_id, team) in teams {
        for channel_id in &team.slack_channel_ids {
            let team_id = team_id.clone();
            let channel = channel_id.clone();
            futures.push(async move {
                let result = check_slack_id_exists(&channel, client).await;
                (team_id, channel, result)
            });
        }
    }

    while let Some((team_id, channel_id, result)) = futures.next().await {
        match result {
            Ok(true) => {}
            Ok(false) => errors.push(ValidationError {
                file: format!("teams/{}.toml", team_id),
                message: format!(
                    "Slack channel ID does not exist: {}",
                    channel_id.red().bold()
                ),
            }),
            Err(e) => warnings.push(ValidationWarning {
                file: format!("teams/{}.toml", team_id),
                message: format!(
                    "Failed to check Slack channel ID {}: {}",
                    channel_id.yellow().bold(),
                    e
                ),
            }),
        }
    }

    (errors, warnings)
}
