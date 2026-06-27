use std::collections::{BTreeMap, HashMap, HashSet};

use governance_schema::team::Feature;

use crate::keycloak::KeycloakClient;
use crate::loader::GovernanceData;

// PostHog organization membership levels
pub const LEVEL_MEMBER: i64 = 1;
pub const LEVEL_OWNER: i64 = 15;

// ScottyLabs is on PostHog US cloud
const HOST: &str = "https://us.posthog.com";

// devops are owners (true), posthog-project leads are members (false), existing owner excluded
pub fn invite_roster(data: &GovernanceData) -> BTreeMap<String, bool> {
    let mut roster: BTreeMap<String, bool> = BTreeMap::new();

    for team in &data.teams {
        let group = &team.team.group;

        if group.slug == "devops" {
            for user in group.all_members() {
                roster.insert(user.to_string(), true);
            }
            for project in &team.team.projects {
                for user in project.group.all_members() {
                    roster.insert(user.to_string(), true);
                }
            }
            continue;
        }

        if group.repos.iter().any(|r| r.has(Feature::Posthog)) {
            for lead in &group.leads {
                roster.entry(lead.clone()).or_insert(false);
            }
        }
        for project in &team.team.projects {
            if !project.group.repos.iter().any(|r| r.has(Feature::Posthog)) {
                continue;
            }
            for lead in group.leads.iter().chain(project.group.leads.iter()) {
                roster.entry(lead.clone()).or_insert(false);
            }
        }
    }

    roster.remove(&data.org.org.tech_director);
    roster
}

// Resolve roster emails through keycloak then invite anyone missing from the org
pub fn reconcile_invites(data: &GovernanceData, dry_run: bool) -> Result<(), String> {
    let roster = invite_roster(data);
    if roster.is_empty() {
        eprintln!("posthog invite roster is empty");
        return Ok(());
    }

    let keycloak = data
        .org
        .org
        .keycloak
        .as_ref()
        .ok_or("keycloak not configured in org.toml")?;
    let client_id = env_nonempty("KEYCLOAK_CLIENT_ID").ok_or("KEYCLOAK_CLIENT_ID not set")?;
    let client_secret =
        env_nonempty("KEYCLOAK_CLIENT_SECRET").ok_or("KEYCLOAK_CLIENT_SECRET not set")?;
    let kc = KeycloakClient::connect(&keycloak.url, &keycloak.realm, &client_id, &client_secret)?;
    let forgejo_url = data
        .org
        .org
        .forgejo
        .as_ref()
        .map(|f| f.url().to_string())
        .unwrap_or_default();

    let client = PosthogClient::from_env()?;
    let members = client.member_levels()?;
    let pending = client.pending_invite_emails()?;

    let mut sent = 0;
    for (username, is_owner) in &roster {
        let level = if *is_owner { LEVEL_OWNER } else { LEVEL_MEMBER };
        let links = kc.lookup_identity_links(username, &forgejo_url)?;
        let Some(saml) = links.get("cmu-saml").filter(|s| !s.is_empty()) else {
            eprintln!("warning: {username} has no cmu-saml link, skipping");
            continue;
        };
        let email = format!("{saml}@andrew.cmu.edu");
        let key = email.to_lowercase();

        if let Some(current) = members.get(&key) {
            if *current < level {
                eprintln!(
                    "note: {email} is a member at level {current} but should be {level}, promote by hand"
                );
            }
            continue;
        }
        if pending.contains(&key) {
            continue;
        }
        if dry_run {
            eprintln!("would invite {email} at level {level}");
        } else {
            client.create_invite(&email, level)?;
            eprintln!("invited {email} at level {level}");
            sent += 1;
        }
    }

    eprintln!("posthog invite reconcile complete, {sent} sent");
    Ok(())
}

// PostHog organization API client for invite reconciliation
pub struct PosthogClient {
    org_id: String,
    token: String,
}

impl PosthogClient {
    // Build from env, error when creds are absent
    pub fn from_env() -> Result<Self, String> {
        let token = env_nonempty("POSTHOG_TOKEN").ok_or("POSTHOG_TOKEN not set")?;
        let org_id =
            env_nonempty("POSTHOG_ORGANIZATION_ID").ok_or("POSTHOG_ORGANIZATION_ID not set")?;
        Ok(Self { org_id, token })
    }

    // Collect every page of an org subresource, following the cursor
    fn paginate(&self, resource: &str) -> Result<Vec<serde_json::Value>, String> {
        let mut out = Vec::new();
        let mut url = format!(
            "{}/api/organizations/{}/{resource}/?limit=200",
            HOST, self.org_id
        );

        loop {
            let mut resp = ureq::get(&url)
                .header("Authorization", &format!("Bearer {}", self.token))
                .call()
                .map_err(|e| format!("get {resource}: {e}"))?;
            let body: serde_json::Value = resp
                .body_mut()
                .read_json()
                .map_err(|e| format!("parse {resource}: {e}"))?;

            if let Some(results) = body["results"].as_array() {
                out.extend(results.iter().cloned());
            }
            match body["next"].as_str() {
                Some(next) if !next.is_empty() => url = next.to_string(),
                _ => break,
            }
        }

        Ok(out)
    }

    // Map each member email lowercased to their org level
    pub fn member_levels(&self) -> Result<HashMap<String, i64>, String> {
        let mut levels = HashMap::new();
        for member in self.paginate("members")? {
            if let Some(email) = member["user"]["email"].as_str() {
                let level = member["level"].as_i64().unwrap_or(LEVEL_MEMBER);
                levels.insert(email.to_lowercase(), level);
            }
        }
        Ok(levels)
    }

    // Lowercased target emails of invites awaiting acceptance
    pub fn pending_invite_emails(&self) -> Result<HashSet<String>, String> {
        let mut emails = HashSet::new();
        for invite in self.paginate("invites")? {
            // Expired invites no longer count as pending, let them be re-sent
            if invite["is_expired"].as_bool() == Some(true) {
                continue;
            }
            if let Some(email) = invite["target_email"].as_str() {
                emails.insert(email.to_lowercase());
            }
        }
        Ok(emails)
    }

    // Send an invite at the given level
    pub fn create_invite(&self, email: &str, level: i64) -> Result<(), String> {
        ureq::post(&format!(
            "{}/api/organizations/{}/invites/",
            HOST, self.org_id
        ))
        .header("Authorization", &format!("Bearer {}", self.token))
        .send_json(serde_json::json!({ "target_email": email, "level": level }))
        .map_err(|e| format!("invite {email}: {e}"))?;
        Ok(())
    }
}

// Env var read that treats empty as absent
fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}
