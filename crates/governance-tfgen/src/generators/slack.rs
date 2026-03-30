use governance_core::loader::GovernanceData;
use governance_schema::team::TeamFile;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    if data.org.org.communication.as_ref().is_none_or(|c| c.slack_workspace.is_empty()) {
        return tf;
    }

    // Usergroup for all tech members, assigned to hub channel
    let hub_channel = &data.org.org.communication.as_ref().unwrap().slack_hub_channel_id;
    let all_member_ids: Vec<String> = data
        .all_members()
        .iter()
        .map(|u| format!("${{data.external.identity_{}.result.slack}}", u.replace('-', "_")))
        .collect();

    tf.add_resource(
        "slack_usergroup",
        "tech",
        json!({
            "handle": "tech",
            "name": "Tech",
        }),
    );

    tf.add_resource(
        "slack_usergroup_members",
        "tech",
        json!({
            "usergroup_id": "${slack_usergroup.tech.id}",
            "members": all_member_ids,
        }),
    );

    tf.add_resource(
        "slack_usergroup_channels",
        "tech",
        json!({
            "usergroup_id": "${slack_usergroup.tech.id}",
            "channels": [hub_channel],
        }),
    );

    // Per-team usergroups
    for team in &data.teams {
        let slug = &team.team.group.slug;
        let name = team.team.group.name.as_deref().unwrap_or(slug);

        let member_ids: Vec<String> = team
            .team
            .group
            .all_members()
            .chain(team.team.projects.iter().flat_map(|p| p.group.all_members()))
            .map(|u| format!("${{data.external.identity_{}.result.slack}}", u.replace('-', "_")))
            .collect();

        let channel_ids: Vec<&str> = slack_channel_ids(team);

        tf.add_resource(
            "slack_usergroup",
            slug,
            json!({
                "handle": slug,
                "name": name,
            }),
        );

        tf.add_resource(
            "slack_usergroup_members",
            slug,
            json!({
                "usergroup_id": format!("${{slack_usergroup.{slug}.id}}"),
                "members": member_ids,
            }),
        );

        if !channel_ids.is_empty() {
            tf.add_resource(
                "slack_usergroup_channels",
                slug,
                json!({
                    "usergroup_id": format!("${{slack_usergroup.{slug}.id}}"),
                    "channels": channel_ids,
                }),
            );
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
