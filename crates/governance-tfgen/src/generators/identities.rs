use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_identity_data_sources(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    let mut usernames = data.all_members();
    usernames.push(data.org.org.tech_director.as_str());
    usernames.sort_unstable();
    usernames.dedup();

    for username in usernames {
        let key = username.replace('-', "_");
        tf.add_data(
            "external",
            &format!("identity_{key}"),
            json!({
                "program": ["governance", "--data-dir", "../../data", "resolve-identity"],
                "query": {
                    "forgejo_user": username,
                },
            }),
        );
    }

    tf
}
