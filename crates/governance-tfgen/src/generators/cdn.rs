use governance_core::loader::GovernanceData;
use governance_schema::team::Feature;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        for repo in team.team.repos().filter(|r| r.has(Feature::Cdn)) {
            let key = format!("{}_{}", team.team.group.slug, repo.name.replace('-', "_"));
            let bucket = format!("cdn-{}", repo.name);

            tf.add_resource(
                "garage_bucket",
                &format!("cdn_{key}"),
                json!({
                    "global_alias": bucket,
                    "website_enabled": true,
                    "website_index_document": "index.html",
                }),
            );

            tf.add_resource(
                "garage_key",
                &format!("cdn_{key}"),
                json!({ "name": bucket }),
            );

            tf.add_resource(
                "garage_bucket_permission",
                &format!("cdn_{key}"),
                json!({
                    "bucket_id": format!("${{garage_bucket.cdn_{key}.id}}"),
                    "access_key_id": format!("${{garage_key.cdn_{key}.id}}"),
                    "read": true,
                    "write": true,
                    "owner": true,
                }),
            );

            for (name, value) in [
                ("CDN_S3_ENDPOINT", "var.garage_s3_endpoint".to_string()),
                ("CDN_S3_BUCKET", format!("\"{bucket}\"")),
                ("CDN_ACCESS_KEY_ID", format!("garage_key.cdn_{key}.id")),
                (
                    "CDN_SECRET_ACCESS_KEY",
                    format!("garage_key.cdn_{key}.secret_access_key"),
                ),
                (
                    "CDN_PUBLIC_URL",
                    format!("\"${{var.cdn_base_url}}/{}/\"", repo.name),
                ),
            ] {
                tf.add_resource(
                    "vault_kv_secret_v2",
                    &format!("{key}_{}", name.to_lowercase()),
                    json!({
                        "mount": "secret",
                        "name": format!("secretspec/{}/prod/{name}", repo.name),
                        "data_json": format!("${{jsonencode({{ value = {value} }})}}"),
                    }),
                );
            }
        }
    }

    tf
}
