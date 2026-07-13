use governance_core::loader::GovernanceData;
use serde_json::json;

use crate::tf_json::TfJsonFile;

pub fn generate(data: &GovernanceData) -> TfJsonFile {
    let mut tf = TfJsonFile::default();

    for team in &data.teams {
        let gateway_repos: Vec<_> = team
            .team
            .repos()
            .filter_map(|r| r.features.ai_gateway.as_ref().map(|g| (r, g)))
            .collect();
        if gateway_repos.is_empty() {
            continue;
        }

        // one team per governance team for spend attribution, budgets enforce on keys
        let team_slug = &team.team.group.slug;
        let team_key = team_slug.replace('-', "_");

        tf.add_resource(
            "litellm_team",
            &team_key,
            json!({ "team_alias": team_slug }),
        );

        for (repo, gateway) in gateway_repos {
            let key = repo.name.replace('-', "_");

            for (env, budget) in [
                ("prod", gateway.prod_monthly_budget),
                ("dev", gateway.dev_monthly_budget),
            ] {
                tf.add_resource(
                    "litellm_key",
                    &format!("{key}_{env}"),
                    json!({
                        "key_alias": format!("{}-{env}", repo.name),
                        "team_id": format!("${{litellm_team.{team_key}.id}}"),
                        "max_budget": budget,
                        "budget_duration": "monthly",
                        "tags": [team_slug, &repo.name],
                        "metadata": { "project": repo.name },
                    }),
                );
            }

            // staging, preview, and dev share the low-budget key
            for profile in ["prod", "staging", "preview", "dev"] {
                let source = if profile == "prod" { "prod" } else { "dev" };

                tf.add_resource(
                    "vault_kv_secret_v2",
                    &format!("{key}_{profile}_litellm_api_key"),
                    json!({
                        "mount": "secret",
                        "name": format!("secretspec/{}/{profile}/LITELLM_API_KEY", repo.name),
                        "data_json": format!(
                            "${{jsonencode({{ value = litellm_key.{key}_{source}.key }})}}"
                        ),
                    }),
                );

                tf.add_resource(
                    "vault_kv_secret_v2",
                    &format!("{key}_{profile}_litellm_base_url"),
                    json!({
                        "mount": "secret",
                        "name": format!("secretspec/{}/{profile}/LITELLM_BASE_URL", repo.name),
                        "data_json": "${jsonencode({ value = var.litellm_url })}",
                    }),
                );
            }
        }
    }

    tf
}
