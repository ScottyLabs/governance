use std::collections::HashSet;
use std::sync::Mutex;
use std::thread;

use governance_schema::org::ForgeType;
use governance_schema::team::{Channel, GroupFields, TeamFile};

use crate::error::ValidationError;
use crate::loader::GovernanceData;

pub fn validate(data: &GovernanceData) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    validate_default_forge(data, &mut errors);
    validate_team_slugs(data, &mut errors);
    validate_repo_names(data, &mut errors);
    validate_groups(data, &mut errors);
    validate_forge_refs(data, &mut errors);
    validate_channels(data, &mut errors);
    validate_identities(data, &mut errors);

    errors
}

fn validate_default_forge(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let org = &data.org.org;
    match org.default_forge {
        ForgeType::Github if org.github.is_none() => {
            errors.push(ValidationError::ForgeNotConfigured("github".into()));
        }
        ForgeType::Forgejo if org.forgejo.is_none() => {
            errors.push(ValidationError::ForgeNotConfigured("forgejo".into()));
        }
        _ => {}
    }
}

fn validate_team_slugs(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let mut seen = HashSet::new();
    for team in &data.teams {
        if !seen.insert(&team.team.group.slug) {
            errors.push(ValidationError::DuplicateTeamSlug(
                team.team.group.slug.clone(),
            ));
        }
    }
}

fn validate_repo_names(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let mut seen = HashSet::new();
    for team in &data.teams {
        for repo in all_repos(team) {
            if !seen.insert(&repo.name) {
                errors.push(ValidationError::DuplicateRepoName(repo.name.clone()));
            }
        }
    }
}

fn validate_groups(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    for team in &data.teams {
        validate_leads_not_members(&team.team.group, errors);
        for project in &team.team.projects {
            validate_leads_not_members(&project.group, errors);
        }
    }
}

fn validate_leads_not_members(group: &GroupFields, errors: &mut Vec<ValidationError>) {
    let leads: HashSet<&str> = group.leads.iter().map(|s| s.as_str()).collect();
    for member in &group.members {
        if leads.contains(member.as_str()) {
            errors.push(ValidationError::LeadAlsoMember {
                team: group.slug.clone(),
                lead: member.clone(),
            });
        }
    }
}

fn validate_forge_refs(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let org = &data.org.org;
    for team in &data.teams {
        let slug = &team.team.group.slug;
        for repo in all_repos(team) {
            if let Some(forge) = &repo.forge {
                let configured = match forge {
                    ForgeType::Github => org.github.is_some(),
                    ForgeType::Forgejo => org.forgejo.is_some(),
                };
                if !configured {
                    errors.push(ValidationError::ForgeNotConfigured(
                        format!("{forge:?} (referenced by repo {} in team {slug})", repo.name),
                    ));
                }
            }
        }
    }
}

fn validate_channels(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let channels = all_channels(data);
    if channels.is_empty() {
        return;
    }

    let has_discord = channels.iter().any(|c| c.discord.is_some());
    let has_slack = channels.iter().any(|c| c.slack.is_some());

    let discord_token = std::env::var("DISCORD_BOT_TOKEN").ok();
    let slack_token = std::env::var("SLACK_BOT_TOKEN").ok();

    if has_discord && discord_token.is_none() {
        errors.push(ValidationError::MissingDiscordToken);
    }
    if has_slack && slack_token.is_none() {
        errors.push(ValidationError::MissingSlackToken);
    }
    if (has_discord && discord_token.is_none()) || (has_slack && slack_token.is_none()) {
        return;
    }

    let collected_errors: Mutex<Vec<ValidationError>> = Mutex::new(Vec::new());

    thread::scope(|s| {
        for channel in &channels {
            if let (Some(id), Some(token)) = (&channel.discord, &discord_token) {
                let id = id.clone();
                let token = token.clone();
                let errs = &collected_errors;
                s.spawn(move || {
                    if let Err(e) = check_discord_channel(&id, &token) {
                        errs.lock().unwrap().push(e);
                    }
                });
            }
            if let (Some(id), Some(token)) = (&channel.slack, &slack_token) {
                let id = id.clone();
                let token = token.clone();
                let errs = &collected_errors;
                s.spawn(move || {
                    if let Err(e) = check_slack_channel(&id, &token) {
                        errs.lock().unwrap().push(e);
                    }
                });
            }
        }
    });

    errors.extend(collected_errors.into_inner().unwrap());
}

fn check_discord_channel(channel_id: &str, token: &str) -> Result<(), ValidationError> {
    let url = format!("https://discord.com/api/v10/channels/{channel_id}");
    let response = ureq::get(&url)
        .header("Authorization", &format!("Bot {token}"))
        .call();

    match response {
        Ok(_) => Ok(()),
        Err(ureq::Error::StatusCode(400 | 404)) => {
            Err(ValidationError::DiscordChannelNotFound {
                channel_id: channel_id.to_string(),
            })
        }
        Err(e) => Err(ValidationError::DiscordApiError(format!(
            "channel {channel_id}: {e}"
        ))),
    }
}

fn check_slack_channel(channel_id: &str, token: &str) -> Result<(), ValidationError> {
    let url = format!("https://slack.com/api/conversations.info?channel={channel_id}");
    let mut response = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| ValidationError::SlackApiError(format!("channel {channel_id}: {e}")))?;

    let body: serde_json::Value = response.body_mut().read_json().map_err(|e| {
        ValidationError::SlackApiError(format!("failed to parse response: {e}"))
    })?;

    if body["ok"].as_bool() == Some(true) {
        Ok(())
    } else {
        Err(ValidationError::SlackChannelNotFound {
            channel_id: channel_id.to_string(),
        })
    }
}

fn all_repos(team: &TeamFile) -> Vec<&governance_schema::team::Repo> {
    let mut repos: Vec<_> = team.team.group.repos.iter().collect();
    for project in &team.team.projects {
        repos.extend(project.group.repos.iter());
    }
    repos
}

fn all_channels(data: &GovernanceData) -> Vec<&Channel> {
    let mut channels = Vec::new();
    for team in &data.teams {
        channels.extend(team.team.group.channels.iter());
        for project in &team.team.projects {
            channels.extend(project.group.channels.iter());
        }
    }
    channels
}

fn validate_identities(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let keycloak_conf = match &data.org.org.keycloak {
        Some(k) => k,
        None => return,
    };

    let client_id = std::env::var("KEYCLOAK_CLIENT_ID");
    let client_secret = std::env::var("KEYCLOAK_CLIENT_SECRET");
    let (client_id, client_secret) = match (client_id, client_secret) {
        (Ok(id), Ok(secret)) => (id, secret),
        _ => {
            errors.push(ValidationError::MissingKeycloakCredentials);
            return;
        }
    };

    let kc = match crate::keycloak::KeycloakClient::connect(
        &keycloak_conf.url,
        &keycloak_conf.realm,
        &client_id,
        &client_secret,
    ) {
        Ok(kc) => kc,
        Err(e) => {
            errors.push(ValidationError::KeycloakApiError(e));
            return;
        }
    };

    let org = &data.org.org;
    let mut required_providers: Vec<&str> = Vec::new();
    if org.forgejo.is_some() {
        required_providers.push("codeberg");
    }
    if org.github.is_some() {
        required_providers.push("github");
    }
    if org.communication.as_ref().is_some_and(|c| !c.discord_guild_id.is_empty()) {
        required_providers.push("discord");
    }
    if org.communication.as_ref().is_some_and(|c| !c.slack_workspace.is_empty()) {
        required_providers.push("slack");
    }

    if required_providers.is_empty() {
        return;
    }

    let members = data.all_members();
    let collected_errors: Mutex<Vec<ValidationError>> = Mutex::new(Vec::new());

    thread::scope(|s| {
        for username in &members {
            let kc = &kc;
            let required = &required_providers;
            let errs = &collected_errors;

            let forgejo_url = org.forgejo.as_ref().map(|f| f.url().to_string()).unwrap_or_default();
            s.spawn(move || {
                match kc.lookup_identity_links(username, &forgejo_url) {
                    Ok(links) => {
                        for provider in required {
                            if !links.contains_key(*provider) {
                                errs.lock().unwrap().push(
                                    ValidationError::MissingIdentityLink {
                                        user: username.to_string(),
                                        provider: provider.to_string(),
                                    },
                                );
                            }
                        }
                    }
                    Err(e) => {
                        if e.contains("no keycloak user found") {
                            errs.lock().unwrap().push(
                                ValidationError::KeycloakUserNotFound {
                                    user: username.to_string(),
                                },
                            );
                        } else {
                            errs.lock().unwrap().push(
                                ValidationError::KeycloakApiError(e),
                            );
                        }
                    }
                }
            });
        }
    });

    errors.extend(collected_errors.into_inner().unwrap());
}
