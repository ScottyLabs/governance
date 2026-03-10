use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use governance::model::{
    Contributor, EntityKey, HasKeyOrder, Repo, Team, ValidationError, ValidationWarning,
};
use log::info;
use reqwest::{Client, StatusCode};
use serde_json::Value;

/// Validate the key orderings in a TOML file against the expected schema order.
pub fn validate_key_orderings<T: HasKeyOrder>(
    data: &HashMap<EntityKey, T>,
    expected_order: &Vec<String>,
) -> Vec<ValidationError> {
    // Get the kind of the item from the first key
    let kind = match data.keys().next() {
        Some(k) => &k.kind,
        None => {
            return vec![ValidationError {
                file: "N/A".to_string(),
                message: "No data found".into(),
            }];
        }
    };

    // Initialization
    info!("Validating {} key orderings...", kind);
    let mut errors = Vec::new();

    // Validate the key orderings
    for (key, item) in data {
        let actual_order = item.get_key_order();
        if !is_subsequence_in_order(actual_order, expected_order) {
            errors.push(ValidationError {
                message: format!(
                    "Invalid key order for {}.\n    - expected (schema): {:?}\n    - found (file): {:?}",
                    key, expected_order, actual_order
                ),
                file: format!("{}s/{}.toml", kind, key),
            });
        }
    }

    errors
}

// Actual order needs to be a subset of the expected order but in the right order
fn is_subsequence_in_order(actual_order: &[String], expected_order: &[String]) -> bool {
    let mut i = 0; // index in expected
    for a in actual_order {
        // advance expected until we find a match
        while i < expected_order.len() && expected_order[i] != *a {
            i += 1;
        }
        if i == expected_order.len() {
            // we ran out of expected items before finding a match
            return false;
        }
        i += 1; // consume that match
    }
    true
}

pub fn validate_file_names(
    contributors: &HashMap<EntityKey, Contributor>,
    teams: &HashMap<EntityKey, Team>,
    repos: &HashMap<EntityKey, Repo>,
) -> Vec<ValidationError> {
    info!("Validating file names...");
    let mut errors = Vec::new();

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

    for (key, team) in teams {
        if key.name != team.slug {
            errors.push(ValidationError {
                file: format!("teams/{}.toml", key),
                message: format!(
                    "Team file name '{}' doesn't match slug '{}'",
                    key.name.red().bold(),
                    team.slug.red().bold()
                ),
            });
        }
    }

    for (key, repo) in repos {
        if key.name != repo.slug {
            errors.push(ValidationError {
                file: format!("repos/{}.toml", key),
                message: format!(
                    "Repo file name '{}' doesn't match slug '{}'",
                    key.name.red().bold(),
                    repo.slug.red().bold()
                ),
            });
        }
    }

    errors
}

fn url_to_owner_repo(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    if let Some(path) = url.strip_prefix("https://github.com/") {
        return Some(path.to_string());
    }
    if let Some(path) = url.strip_prefix("https://codeberg.org/") {
        return Some(path.to_string());
    }
    None
}

pub fn validate_team_repo_refs(
    teams: &HashMap<EntityKey, Team>,
    repos: &HashMap<EntityKey, Repo>,
) -> Vec<ValidationError> {
    info!("Validating team repo references...");
    let mut errors = Vec::new();
    let repo_slugs: HashSet<&String> = repos.values().map(|r| &r.slug).collect();

    for (team_key, team) in teams {
        for repo_ref in &team.repos {
            if !repo_ref.contains('/') {
                if !repo_slugs.contains(repo_ref) {
                    errors.push(ValidationError {
                        file: format!("teams/{}.toml", team_key),
                        message: format!(
                            "Team references repo slug '{}' but no repos/{}.toml exists",
                            repo_ref.red().bold(),
                            repo_ref.red().bold()
                        ),
                    });
                }
            }
        }
    }

    errors
}

pub fn validate_no_duplicate_team_repos(
    teams: &HashMap<EntityKey, Team>,
    repos: &HashMap<EntityKey, Repo>,
) -> Vec<ValidationError> {
    info!("Validating no duplicate repos per team...");
    let mut errors = Vec::new();
    let legacy_to_slug: HashMap<String, String> = repos
        .values()
        .filter_map(|r| url_to_owner_repo(&r.url).map(|legacy| (legacy, r.slug.clone())))
        .collect();

    for (team_key, team) in teams {
        let mut canonical_to_refs: HashMap<String, Vec<&str>> = HashMap::new();
        for ref_ in &team.repos {
            let canonical = if ref_.contains('/') {
                legacy_to_slug
                    .get(ref_.as_str())
                    .cloned()
                    .unwrap_or_else(|| ref_.clone())
            } else {
                ref_.clone()
            };
            canonical_to_refs
                .entry(canonical)
                .or_default()
                .push(ref_.as_str());
        }

        for (_canonical, refs) in canonical_to_refs {
            if refs.len() > 1 {
                let unique_refs: HashSet<&str> = refs.iter().copied().collect();
                let refs_display = unique_refs.iter().map(|r| format!("'{}'", r)).collect::<Vec<_>>().join(" and ");
                errors.push(ValidationError {
                    file: format!("teams/{}.toml", team_key),
                    message: format!(
                        "Duplicate repo: {} refer to the same repository",
                        refs_display.red().bold()
                    ),
                });
            }
        }
    }

    errors
}

pub fn validate_maintainers_are_contributors(
    teams: &HashMap<EntityKey, Team>,
) -> Vec<ValidationError> {
    info!("Validating that all maintainers are also contributors...");
    let mut errors = Vec::new();
    for (team_key, team) in teams {
        let contributors: HashSet<&String> = team.contributors.iter().collect();
        for maintainer in &team.maintainers {
            if !contributors.contains(maintainer) {
                errors.push(ValidationError {
                    file: format!("teams/{}.toml", team_key),
                    message: format!(
                        "Maintainer '{}' is not a contributor",
                        maintainer.red().bold()
                    ),
                });
            }
        }
    }

    errors
}

pub fn validate_cross_references(
    contributors: &HashMap<EntityKey, Contributor>,
    teams: &HashMap<EntityKey, Team>,
) -> Vec<ValidationError> {
    info!("Validating cross-references...");
    let mut errors = Vec::new();

    // Check that all team participants exist in contributors
    for (team_key, team) in teams {
        // Participants are contributors and applicants combined
        let participants = team
            .maintainers
            .iter()
            .chain(team.contributors.iter())
            .chain(team.applicants.iter().flatten())
            .collect::<Vec<_>>();
        for participant in &participants {
            let key = EntityKey {
                kind: "contributor".to_string(),
                name: participant.to_string(),
            };

            if !contributors.contains_key(&key) {
                errors.push(ValidationError {
                    file: format!("teams/{}.toml", team_key),
                    message: format!(
                        "Team '{}' references non-existent contributor: {}",
                        team_key.name.red().bold(),
                        participant.red().bold()
                    ),
                });
            }
        }
    }

    errors
}

async fn check_github_user_exists(github_username: &str, client: &Client) -> Result<bool> {
    // GitHub API has rate limiting, so we use a token from the environment if possible.
    let token = std::env::var("SYNC_GITHUB_TOKEN").unwrap_or_default();

    let request = client
        .get(format!("https://api.github.com/users/{}", github_username))
        .header("User-Agent", "ScottyLabs-Governance-Validator")
        .bearer_auth(token);

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

enum RepoCheckResult {
    ExistsInOrg,
    ExistsOutsideOrg(String),
    NotFound,
}

const SCOTTYLABS_ORG: &str = "ScottyLabs";

async fn check_github_repository_exists(
    repository: &str,
    client: &Client,
) -> Result<RepoCheckResult> {
    // GitHub API has rate limiting, so we use a token from the environment if possible.
    let token = std::env::var("SYNC_GITHUB_TOKEN").unwrap_or_default();

    let request = client
        .get(format!("https://api.github.com/repos/{}", repository))
        .header("User-Agent", "ScottyLabs-Governance-Validator")
        .bearer_auth(token);

    let response = request.send().await?;
    let status = response.status();

    match status {
        StatusCode::OK => {
            // Make sure the repository is in the ScottyLabs organization.
            let json = response.json::<Value>().await?;
            if let Some(org_value) = json["organization"]["login"].as_str() {
                if org_value == "ScottyLabs" {
                    Ok(RepoCheckResult::ExistsInOrg)
                } else {
                    Ok(RepoCheckResult::ExistsOutsideOrg(org_value.to_string()))
                }
            } else {
                Ok(RepoCheckResult::ExistsOutsideOrg("<no org>".to_string()))
            }
        }
        StatusCode::NOT_FOUND => Ok(RepoCheckResult::NotFound),
        StatusCode::FORBIDDEN => Err(anyhow!("Rate limit exceeded or access forbidden",)),
        _ => Err(anyhow!("Unexpected status {}", status,)),
    }
}

pub async fn validate_github_repositories(
    teams: &HashMap<EntityKey, Team>,
    client: &Client,
) -> (Vec<ValidationError>, Vec<ValidationWarning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut futures = FuturesUnordered::new();

    for (team_key, team) in teams {
        for repository in &team.repos {
            if !repository.contains('/') {
                continue;
            }
            let team_key = team_key.clone();
            let repository = repository.clone();
            futures.push(async move {
                let result = check_github_repository_exists(&repository, client).await;
                (team_key, repository, result)
            });
        }
    }

    while let Some((team_key, repository, result)) = futures.next().await {
        match result {
            Ok(RepoCheckResult::ExistsInOrg) => {}
            Ok(RepoCheckResult::ExistsOutsideOrg(organization)) => errors.push(ValidationError {
                file: format!("teams/{}.toml", team_key),
                message: format!(
                    "GitHub repository {} is not in the \"{}\" organization. It is in the {} organization.",
                    repository.red().bold(),
                    SCOTTYLABS_ORG,
                    organization.red().bold()
                ),
            }),
            Ok(RepoCheckResult::NotFound) => errors.push(ValidationError {
                file: format!("teams/{}.toml", team_key),
                message: format!(
                    "GitHub repository does not exist: {}",
                    repository.red().bold()
                ),
            }),
            Err(e) => warnings.push(ValidationWarning {
                file: format!("teams/{}.toml", team_key),
                message: format!(
                    "Failed to check GitHub repository {}: {}",
                    repository.yellow().bold(),
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
        if let Some(slack_id) = &contributor.slack_member_id {
            futures.push(async move {
                let result = check_slack_id_exists(slack_id, client).await;
                (contributor_id, slack_id, result)
            });
        }
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
