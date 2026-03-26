use governance_core::loader::GovernanceData;
use governance_schema::team::TeamFile;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let comm = match data.org.org.communication.as_ref() {
        Some(c) if !c.discord_guild_id.is_empty() => c,
        _ => return tf,
    };
    let guild = &comm.discord_guild_id;

    // "Tech" role for all members
    tf.add_resource(
        "discord_role",
        "tech",
        json!({
            "server_id": guild,
            "name": "Tech",
        }),
    );

    // Tech role gives access to hub channel
    tf.add_resource(
        "discord_channel_permission",
        "tech_hub",
        json!({
            "channel_id": comm.discord_hub_channel_id,
            "type": "role",
            "overwrite_id": "${discord_role.tech.id}",
            "allow": 1024,
        }),
    );

    // All members get the Tech role
    for username in data.all_members() {
        let key = username.replace('-', "_");
        tf.add_resource(
            "discord_member_roles",
            &format!("tech_{key}"),
            json!({
                "server_id": guild,
                "user_id": format!("${{data.external.identity_{key}.result.discord}}"),
                "role": ["${discord_role.tech.id}"],
            }),
        );
    }

    // Per-team roles
    for team in &data.teams {
        let slug = &team.team.group.slug;
        let name = team.team.group.name.as_deref().unwrap_or(slug);

        tf.add_resource(
            "discord_role",
            slug,
            json!({
                "server_id": guild,
                "name": name,
            }),
        );

        // Team role gives access to all team/project channels
        let role_id = format!("${{discord_role.{slug}.id}}");
        let mut ch_idx = 0;
        for channel_id in discord_channel_ids(team) {
            tf.add_resource(
                "discord_channel_permission",
                &format!("{slug}_ch{ch_idx}"),
                json!({
                    "channel_id": channel_id,
                    "type": "role",
                    "overwrite_id": role_id,
                    "allow": 1024,
                }),
            );
            ch_idx += 1;
        }

        // All team members, including project members, get the team role
        let all = team.team.group.all_members().chain(
            team.team
                .projects
                .iter()
                .flat_map(|p| p.group.all_members()),
        );

        for username in all {
            let key = username.replace('-', "_");
            tf.add_resource(
                "discord_member_roles",
                &format!("{slug}_{key}"),
                json!({
                    "server_id": guild,
                    "user_id": format!("${{data.external.identity_{key}.result.discord}}"),
                    "role": [role_id],
                }),
            );
        }
    }

    tf
}

fn discord_channel_ids(team: &TeamFile) -> Vec<&str> {
    let mut ids = Vec::new();
    for channel in &team.team.group.channels {
        if let Some(id) = &channel.discord {
            ids.push(id.as_str());
        }
    }
    for project in &team.team.projects {
        for channel in &project.group.channels {
            if let Some(id) = &channel.discord {
                ids.push(id.as_str());
            }
        }
    }
    ids
}
