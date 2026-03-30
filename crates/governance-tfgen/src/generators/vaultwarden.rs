use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();
    let vw = match &data.org.org.vaultwarden {
        Some(v) => v,
        None => return tf,
    };

    let td = &data.org.org.tech_director;
    let td_key = td.replace('-', "_");

    let devops_leads: Vec<&str> = data
        .teams
        .iter()
        .filter(|t| t.team.group.slug == "devops")
        .flat_map(|t| t.team.group.leads.iter().map(|s| s.as_str()))
        .collect();

    let mut tech_members = vec![json!({
        "id": format!("${{data.external.identity_{td_key}.result.keycloak_user_id}}"),
        "manage": true,
    })];
    for lead in &devops_leads {
        if *lead != td.as_str() {
            let key = lead.replace('-', "_");
            tech_members.push(json!({
                "id": format!("${{data.external.identity_{key}.result.keycloak_user_id}}"),
                "manage": true,
            }));
        }
    }

    tf.add_resource(
        "bitwarden_org_collection",
        "tech",
        json!({
            "organization_id": vw.org_id,
            "name": "Tech",
            "member": tech_members,
        }),
    );

    let lead_members: Vec<serde_json::Value> = data
        .all_leads()
        .iter()
        .map(|username| {
            let key = username.replace('-', "_");
            json!({
                "id": format!("${{data.external.identity_{key}.result.keycloak_user_id}}"),
                "manage": true,
            })
        })
        .collect();

    tf.add_resource(
        "bitwarden_org_collection",
        "tech_leads",
        json!({
            "organization_id": vw.org_id,
            "name": "Tech/Tech Leads",
            "member": lead_members,
        }),
    );

    tf
}
