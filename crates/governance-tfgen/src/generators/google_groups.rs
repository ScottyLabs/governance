use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let gg = match &data.org.org.google_groups {
        Some(g) => g,
        None => return tf,
    };

    let td = &data.org.org.tech_director;

    let devops_leads: Vec<&str> = data
        .teams
        .iter()
        .filter(|t| t.team.group.slug == "devops")
        .flat_map(|t| t.team.group.leads.iter().map(|s| s.as_str()))
        .collect();

    let all_leads = data.all_leads();
    let all_members = data.all_members();

    // sl-admin (tech director + devops leads)
    let mut admin_members: Vec<&str> = vec![td.as_str()];
    for lead in &devops_leads {
        if *lead != td.as_str() {
            admin_members.push(lead);
        }
    }
    emit_group(&mut tf, "admin", &gg.admin, &admin_members);

    // sl-ops (tech director + all leads)
    let mut ops_members: Vec<&str> = vec![td.as_str()];
    for lead in &all_leads {
        if *lead != td.as_str() {
            ops_members.push(lead);
        }
    }
    emit_group(&mut tf, "ops", &gg.ops, &ops_members);

    // sl-tech (everyone)
    emit_group(&mut tf, "tech", &gg.tech, &all_members);

    tf
}

fn emit_group(tf: &mut TfJsonFile, key: &str, email: &str, members: &[&str]) {
    tf.add_data(
        "google_cloud_identity_group_lookup",
        key,
        json!({
            "group_key": {
                "id": email,
            },
        }),
    );

    for username in members {
        let user_key = username.replace('-', "_");
        tf.add_resource(
            "google_cloud_identity_group_membership",
            &format!("{key}_{user_key}"),
            json!({
                "group": format!("${{data.google_cloud_identity_group_lookup.{key}.name}}"),
                "preferred_member_key": {
                    "id": format!("${{data.external.identity_{user_key}.result.cmu-saml}}@andrew.cmu.edu"),
                },
                "roles": [{ "name": "MEMBER" }],
            }),
        );
    }
}
