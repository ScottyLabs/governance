use governance_core::loader::GovernanceData;
use governance_schema::team::TeamFile;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let comm = match data.org.org.communication.as_ref() {
        Some(c) if !c.slack_workspace.is_empty() => c,
        _ => return tf,
    };

    // All members get added to the hub channel
    for username in data.all_members() {
        let key = username.replace('-', "_");
        tf.add_resource(
            "slack_conversation_member",
            &format!("hub_{key}"),
            json!({
                "channel_id": comm.slack_hub_channel_id,
                "user_ids": [format!("${{data.external.identity_{key}.result.slack}}")],
            }),
        );
    }

    // Per-team channel membership
    for team in &data.teams {
        let slug = &team.team.group.slug;

        let all: Vec<&str> = team.team.group.all_members()
            .chain(team.team.projects.iter().flat_map(|p| p.group.all_members()))
            .collect();

        let mut ch_idx = 0;
        for channel_id in slack_channel_ids(team) {
            for username in &all {
                let key = username.replace('-', "_");
                tf.add_resource(
                    "slack_conversation_member",
                    &format!("{slug}_ch{ch_idx}_{key}"),
                    json!({
                        "channel_id": channel_id,
                        "user_ids": [format!("${{data.external.identity_{key}.result.slack}}")],
                    }),
                );
            }
            ch_idx += 1;
        }
    }

    tf
}

fn slack_channel_ids(team: &TeamFile) -> Vec<&str> {
    let mut ids = Vec::new();
    for channel in &team.team.group.channels {
        if let Some(id) = &channel.slack {
            ids.push(id.as_str());
        }
    }
    for project in &team.team.projects {
        for channel in &project.group.channels {
            if let Some(id) = &channel.slack {
                ids.push(id.as_str());
            }
        }
    }
    ids
}
