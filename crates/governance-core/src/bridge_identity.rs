use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::keycloak::KeycloakClient;
use crate::loader::GovernanceData;

#[derive(Serialize)]
pub struct BridgeIdentityMap {
    pub slack_team_id: String,
    pub links: Vec<BridgeIdentityLink>,
}

#[derive(Serialize)]
pub struct BridgeIdentityLink {
    pub discord_id: String,
    pub slack_user_id: String,
}

pub fn generate_bridge_identity_map(data: &GovernanceData) -> Result<BridgeIdentityMap, String> {
    let comm = data
        .org
        .org
        .communication
        .as_ref()
        .ok_or_else(|| "org.communication not configured".to_string())?;

    let keycloak_conf = data
        .org
        .org
        .keycloak
        .as_ref()
        .ok_or_else(|| "org.keycloak not configured".to_string())?;

    let client_id = std::env::var("KEYCLOAK_CLIENT_ID")
        .map_err(|_| "KEYCLOAK_CLIENT_ID not set".to_string())?;
    let client_secret = std::env::var("KEYCLOAK_CLIENT_SECRET")
        .map_err(|_| "KEYCLOAK_CLIENT_SECRET not set".to_string())?;

    let kc = KeycloakClient::connect(
        &keycloak_conf.url,
        &keycloak_conf.realm,
        &client_id,
        &client_secret,
    )?;

    let forgejo_url = data
        .org
        .org
        .forgejo
        .as_ref()
        .map(|f| f.url().to_string())
        .unwrap_or_default();

    let mut links = Vec::new();
    for username in data.all_members() {
        let result = kc.lookup_identity_links(username, &forgejo_url)?;
        let discord_id = result.get("discord_id").or_else(|| result.get("discord"));
        let slack_id = result.get("slack_id").or_else(|| result.get("slack"));
        if let (Some(discord_id), Some(slack_id)) = (discord_id, slack_id) {
            if !discord_id.is_empty() && !slack_id.is_empty() {
                links.push(BridgeIdentityLink {
                    discord_id: discord_id.clone(),
                    slack_user_id: slack_id.clone(),
                });
            }
        }
    }

    links.sort_by(|a, b| a.discord_id.cmp(&b.discord_id));

    Ok(BridgeIdentityMap {
        slack_team_id: comm.slack_team_id.clone(),
        links,
    })
}

pub fn write_bridge_identity_map(data: &GovernanceData, path: &Path) -> Result<(), String> {
    let map = generate_bridge_identity_map(data)?;
    let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Lookup helpers for tests / validation.
pub fn link_maps(links: &[BridgeIdentityLink]) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut discord_to_slack = HashMap::new();
    let mut slack_to_discord = HashMap::new();
    for link in links {
        discord_to_slack.insert(link.discord_id.clone(), link.slack_user_id.clone());
        slack_to_discord.insert(
            link.slack_user_id.to_uppercase(),
            link.discord_id.clone(),
        );
    }
    (discord_to_slack, slack_to_discord)
}
