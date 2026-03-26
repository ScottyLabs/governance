use std::collections::HashMap;

pub struct KeycloakClient {
    url: String,
    realm: String,
    token: String,
}

impl KeycloakClient {
    pub fn connect(
        url: &str,
        realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, String> {
        let token_url = format!("{url}/realms/{realm}/protocol/openid-connect/token");
        let mut resp = ureq::post(&token_url)
            .send_form([
                ("grant_type", "client_credentials"),
                ("client_id", client_id),
                ("client_secret", client_secret),
            ])
            .map_err(|e| format!("token request failed: {e}"))?;

        let body: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse token response: {e}"))?;

        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| "no access_token in response".to_string())?
            .to_string();

        Ok(Self {
            url: url.to_string(),
            realm: realm.to_string(),
            token,
        })
    }

    pub fn lookup_identity_links(
        &self,
        codeberg_user: &str,
        forgejo_url: &str,
    ) -> Result<HashMap<String, String>, String> {
        let codeberg_id = resolve_forgejo_user_id(forgejo_url, codeberg_user)?;
        let users_url = format!(
            "{}/admin/realms/{}/users?idpAlias=codeberg&idpUserId={codeberg_id}",
            self.url, self.realm
        );
        let mut resp = ureq::get(&users_url)
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .map_err(|e| format!("user lookup for {codeberg_user}: {e}"))?;

        let users: Vec<serde_json::Value> = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("parse error: {e}"))?;

        let user = users
            .first()
            .ok_or_else(|| format!("no keycloak user found for codeberg user {codeberg_user}"))?;

        let user_id = user["id"]
            .as_str()
            .ok_or_else(|| "user has no id".to_string())?;

        let links_url = format!(
            "{}/admin/realms/{}/users/{user_id}/federated-identity",
            self.url, self.realm
        );
        let mut links_resp = ureq::get(&links_url)
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .map_err(|e| format!("identity links for {codeberg_user}: {e}"))?;

        let links: Vec<serde_json::Value> = links_resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("parse error: {e}"))?;

        Ok(links
            .iter()
            .filter_map(|l| {
                let provider = l["identityProvider"].as_str()?;
                let username = l["userName"].as_str()?;
                Some((provider.to_string(), username.to_string()))
            })
            .collect())
    }
}

fn resolve_forgejo_user_id(forgejo_url: &str, username: &str) -> Result<String, String> {
    let api_url = format!("{forgejo_url}/api/v1/users/{username}");
    let mut resp = ureq::get(&api_url)
        .call()
        .map_err(|e| format!("codeberg user lookup for {username}: {e}"))?;

    let body: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| format!("parse error: {e}"))?;

    body["id"]
        .as_i64()
        .map(|id| id.to_string())
        .ok_or_else(|| format!("codeberg user {username} has no id"))
}
