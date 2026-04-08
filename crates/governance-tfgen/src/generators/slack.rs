use governance_core::loader::GovernanceData;
use governance_schema::team::TeamFile;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.communication.as_ref().is_none_or(|c| c.slack_workspace.is_empty()) {
        return tf;
    }

    let hub_channel = &data.org.org.communication.as_ref().unwrap().slack_hub_channel_id;

    for username in data.all_members() {
        let key = username.replace('-', "_");
        let slack_id = format!("${{data.external.identity_{key}.result.slack_id}}");

        emit_channel_invite(&mut tf, &format!("slack_hub_{key}"), hub_channel, &slack_id);
    }

    for team in &data.teams {
        let slug = &team.team.group.slug;
        let channel_ids = slack_channel_ids(team);

        let all: Vec<&str> = team
            .team
            .group
            .all_members()
            .chain(team.team.projects.iter().flat_map(|p| p.group.all_members()))
            .collect();

        for (ch_idx, channel_id) in channel_ids.iter().enumerate() {
            for username in &all {
                let key = username.replace('-', "_");
                let slack_id = format!("${{data.external.identity_{key}.result.slack_id}}");

                emit_channel_invite(
                    &mut tf,
                    &format!("slack_{slug}_ch{ch_idx}_{key}"),
                    channel_id,
                    &slack_id,
                );
            }
        }
    }

    tf
}

fn emit_channel_invite(tf: &mut TfJsonFile, resource_name: &str, channel: &str, user: &str) {
    tf.add_resource(
        "null_resource",
        resource_name,
        json!({
            "triggers": {
                "channel": channel,
                "user": user,
            },
            "provisioner": [
                {
                    "local-exec": {
                        "command": "governance slack-invite --channel ${self.triggers.channel} --user ${self.triggers.user}",
                    },
                },
                {
                    "local-exec": {
                        "when": "destroy",
                        "command": "governance slack-kick --channel ${self.triggers.channel} --user ${self.triggers.user}",
                    },
                },
            ],
        }),
    );
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
