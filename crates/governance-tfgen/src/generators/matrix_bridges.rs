use governance_core::loader::GovernanceData;
use governance_schema::org::OrgChannel;
use governance_schema::team::Channel;
use serde_json::json;

use crate::tf_json::TfJsonFile;

const ORG_SLUG: &str = "org";

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    emit_matrix_locals(&mut tf, data);
    emit_org_bridges(&mut tf, data);

    for team in &data.teams {
        let group = &team.team.group;
        emit_channel_pairs(
            &mut tf,
            &group.name,
            &group.slug,
            None,
            None,
            &group.channels,
        );

        for project in &team.team.projects {
            let project_group = &project.group;
            emit_channel_pairs(
                &mut tf,
                &group.name,
                &group.slug,
                Some(project_group.name.as_str()),
                Some(project_group.slug.as_str()),
                &project_group.channels,
            );
        }
    }

    tf
}

fn emit_matrix_locals(tf: &mut TfJsonFile, data: &GovernanceData) {
    let Some(comm) = data.org.org.communication.as_ref() else {
        return;
    };
    tf.add_local("matrix_slack_team_id", json!(comm.slack_team_id));
}

fn emit_org_bridges(tf: &mut TfJsonFile, data: &GovernanceData) {
    let Some(comm) = data.org.org.communication.as_ref() else {
        return;
    };
    let org_name = &data.org.org.name;

    emit_org_pair(
        tf,
        org_name,
        "Hub",
        "hub",
        &comm.slack_hub_channel_id,
        &comm.discord_hub_channel_id,
    );

    // Tech leads channel
    emit_org_pair(
        tf,
        org_name,
        "Tech Leads",
        "leads",
        &comm.slack_leads_channel_id,
        &comm.discord_leads_channel_id,
    );

    for channel in &comm.channels {
        emit_org_channel(tf, org_name, channel);
    }
}

fn emit_org_pair(
    tf: &mut TfJsonFile,
    org_name: &str,
    channel_name: &str,
    channel_slug: &str,
    slack: &str,
    discord: &str,
) {
    emit_channel_pairs(
        tf,
        org_name,
        ORG_SLUG,
        Some(channel_name),
        Some(channel_slug),
        &[Channel {
            slack: Some(slack.to_string()),
            discord: Some(discord.to_string()),
        }],
    );
}

fn emit_org_channel(tf: &mut TfJsonFile, org_name: &str, channel: &OrgChannel) {
    emit_channel_pairs(
        tf,
        org_name,
        ORG_SLUG,
        Some(channel.name.as_str()),
        Some(channel.slug.as_str()),
        &[Channel {
            slack: channel.slack.clone(),
            discord: channel.discord.clone(),
        }],
    );
}

fn emit_channel_pairs(
    tf: &mut TfJsonFile,
    team_name: &str,
    team_slug: &str,
    project_name: Option<&str>,
    project_slug: Option<&str>,
    channels: &[Channel],
) {
    for channel in channels {
        if let (Some(slack), Some(discord)) = (&channel.slack, &channel.discord) {
            let resource_name = resource_name(team_slug, project_slug);
            let invite_name = format!("slack_bridge_login_{resource_name}");
            emit_relay_login_invite(tf, &invite_name, slack);

            let mut body = json!({
                "discord_channel_id": discord,
                "slack_channel_id": slack,
                "team_name": team_name,
                "team_slug": team_slug,
                "depends_on": [format!("null_resource.{invite_name}")],
            });
            if let Some(name) = project_name {
                body["project_name"] = json!(name);
            }
            if let Some(slug) = project_slug {
                body["project_slug"] = json!(slug);
            }
            tf.add_resource("synapse_mautrix_slack_link", &resource_name, body);
        }
    }
}

fn resource_name(team_slug: &str, project_slug: Option<&str>) -> String {
    let raw = match project_slug {
        Some(slug) => format!("{team_slug}_{slug}"),
        None => team_slug.to_string(),
    };
    raw.replace('-', "_")
}

fn emit_relay_login_invite(tf: &mut TfJsonFile, resource_name: &str, channel: &str) {
    tf.add_resource(
        "null_resource",
        resource_name,
        json!({
            "triggers": {
                "channel": channel,
            },
            "provisioner": [
                {
                    "local-exec": {
                        "command": "governance slack-join --channel ${self.triggers.channel}",
                    },
                },
                {
                    "local-exec": {
                        "when": "destroy",
                        "command": "governance slack-leave --channel ${self.triggers.channel}",
                    },
                },
            ],
        }),
    );
}
