use serde_json::json;

pub fn invite(channel: &str, user: &str) -> Result<(), String> {
    let token = token()?;
    // Best-effort self-join so a freshly authed relay user can invite others
    let _ = post(&token, "conversations.join", &json!({ "channel": channel }));
    let body = post(
        &token,
        "conversations.invite",
        &json!({ "channel": channel, "users": user }),
    )?;
    check(&body, "invite", channel, &["already_in_channel"])
}

pub fn kick(channel: &str, user: &str) -> Result<(), String> {
    let token = token()?;
    let body = post(
        &token,
        "conversations.kick",
        &json!({ "channel": channel, "user": user }),
    )?;
    check(&body, "kick", channel, &["not_in_channel"])
}

pub fn join(channel: &str) -> Result<(), String> {
    let token = token()?;
    let info = get(&token, "conversations.info", &[("channel", channel)])?;
    check(&info, "info", channel, &[])?;
    if info["channel"]["is_member"].as_bool() == Some(true) {
        return Ok(());
    }
    if info["channel"]["is_private"].as_bool() == Some(true) {
        return Err(format!(
            "channel {channel} is private; the slack relay login must be invited manually by an existing member"
        ));
    }
    let body = post(&token, "conversations.join", &json!({ "channel": channel }))?;
    check(&body, "join", channel, &["already_in_channel"])
}

pub fn leave(channel: &str) -> Result<(), String> {
    let token = token()?;
    let body = post(
        &token,
        "conversations.leave",
        &json!({ "channel": channel }),
    )?;
    check(&body, "leave", channel, &["not_in_channel"])
}

fn token() -> Result<String, String> {
    std::env::var("SLACK_TOKEN").map_err(|_| "SLACK_TOKEN not set".to_string())
}

fn post(token: &str, method: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    ureq::post(&format!("https://slack.com/api/{method}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send_json(body)
        .map_err(|e| format!("slack {method} request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("slack {method} response read failed: {e}"))
}

fn get(token: &str, method: &str, query: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let mut req = ureq::get(&format!("https://slack.com/api/{method}"))
        .header("Authorization", &format!("Bearer {token}"));
    for (k, v) in query {
        req = req.query(*k, *v);
    }
    req.call()
        .map_err(|e| format!("slack {method} request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("slack {method} response read failed: {e}"))
}

fn check(
    body: &serde_json::Value,
    label: &str,
    channel: &str,
    tolerate: &[&str],
) -> Result<(), String> {
    if body["ok"].as_bool() == Some(true) {
        return Ok(());
    }
    let err = body["error"].as_str().unwrap_or("unknown");
    if tolerate.contains(&err) {
        return Ok(());
    }
    Err(format!("slack {label} {channel}: {err}"))
}
