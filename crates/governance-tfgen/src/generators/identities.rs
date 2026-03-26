use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate_identity_data_sources(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for username in data.all_members() {
        let key = username.replace('-', "_");
        tf.add_data(
            "external",
            &format!("identity_{key}"),
            json!({
                "program": ["governance", "resolve-identity"],
                "query": {
                    "codeberg_user": username,
                },
            }),
        );
    }

    tf
}
