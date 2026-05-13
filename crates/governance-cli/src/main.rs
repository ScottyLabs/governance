use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use governance_core::loader::GovernanceData;
use governance_core::validator;
use governance_tfgen::codeowners;
use governance_tfgen::generators::{
    discord, forgejo, github, google_groups, identities, keycloak, openbao, sentry, slack,
    vaultwarden,
};

#[derive(Parser)]
#[command(name = "governance", about = "ScottyLabs governance CLI")]
struct Cli {
    #[arg(long, default_value = "data")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Validate,
    Generate {
        #[arg(long, default_value = "tofu")]
        output_dir: PathBuf,
    },
    Schema {
        #[arg(long, default_value = "schemas")]
        output_dir: PathBuf,
    },
    ResolveIdentity,
    CheckPr {
        #[arg(long)]
        author: String,
        #[arg(long, default_value = "main")]
        base_ref: String,
        #[arg(long, value_delimiter = ',')]
        changed_files: Vec<String>,
    },
    SlackInvite {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        user: String,
    },
    SlackKick {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        user: String,
    },
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Command::Validate => {
            let data = GovernanceData::load(&cli.data_dir)?;
            let errors = validator::validate(&data);
            if errors.is_empty() {
                eprintln!("validation passed");
            } else {
                for err in &errors {
                    eprintln!("error: {err}");
                }
                process::exit(1);
            }
        }
        Command::Generate { output_dir } => {
            let data = GovernanceData::load(&cli.data_dir)?;
            std::fs::create_dir_all(&output_dir)?;

            std::fs::create_dir_all(".forgejo")?;
            let codeowners_content = codeowners::generate_codeowners(&data);
            std::fs::write(".forgejo/CODEOWNERS", codeowners_content)?;
            eprintln!("wrote .forgejo/CODEOWNERS");

            identities::generate_identity_data_sources(&data)
                .write_to(&output_dir.join("identities.tf.json"))?;

            forgejo::generate_repos(&data).write_to(&output_dir.join("forgejo_repos.tf.json"))?;
            forgejo::generate_teams(&data).write_to(&output_dir.join("forgejo_teams.tf.json"))?;
            forgejo::generate_team_memberships(&data)
                .write_to(&output_dir.join("forgejo_memberships.tf.json"))?;
            forgejo::generate_push_mirrors(&data)
                .write_to(&output_dir.join("forgejo_push_mirrors.tf.json"))?;
            forgejo::generate_kennel_webhooks(&data)
                .write_to(&output_dir.join("forgejo_kennel_webhooks.tf.json"))?;

            keycloak::generate_groups(&data)
                .write_to(&output_dir.join("keycloak_groups.tf.json"))?;
            keycloak::generate_group_memberships(&data)
                .write_to(&output_dir.join("keycloak_memberships.tf.json"))?;

            openbao::generate_project_policies(&data)
                .write_to(&output_dir.join("openbao.tf.json"))?;

            sentry::generate(&data).write_to(&output_dir.join("sentry.tf.json"))?;

            vaultwarden::generate(&data).write_to(&output_dir.join("vaultwarden.tf.json"))?;
            google_groups::generate(&data).write_to(&output_dir.join("google_groups.tf.json"))?;
            discord::generate(&data).write_to(&output_dir.join("discord.tf.json"))?;
            slack::generate(&data).write_to(&output_dir.join("slack.tf.json"))?;

            github::generate_repos(&data).write_to(&output_dir.join("github_repos.tf.json"))?;
            github::generate_teams(&data).write_to(&output_dir.join("github_teams.tf.json"))?;
            github::generate_team_memberships(&data)
                .write_to(&output_dir.join("github_memberships.tf.json"))?;

            eprintln!("wrote {}", output_dir.display());
        }
        Command::Schema { output_dir } => {
            std::fs::create_dir_all(&output_dir)?;

            let org_schema = schemars::schema_for!(governance_schema::org::OrgFile);
            std::fs::write(
                output_dir.join("org.schema.json"),
                serde_json::to_string_pretty(&org_schema)?,
            )?;

            let team_schema = schemars::schema_for!(governance_schema::team::TeamFile);
            std::fs::write(
                output_dir.join("team.schema.json"),
                serde_json::to_string_pretty(&team_schema)?,
            )?;

            eprintln!("wrote schemas to {}", output_dir.display());
        }
        Command::ResolveIdentity => {
            let query: serde_json::Value = serde_json::from_reader(std::io::stdin())?;
            let codeberg_user = query["codeberg_user"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("missing codeberg_user in query"))?;

            let data = GovernanceData::load(&cli.data_dir)?;
            let keycloak_conf = data
                .org
                .org
                .keycloak
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("keycloak not configured in org.toml"))?;

            let client_id = std::env::var("KEYCLOAK_CLIENT_ID")
                .map_err(|_| anyhow::anyhow!("KEYCLOAK_CLIENT_ID not set"))?;
            let client_secret = std::env::var("KEYCLOAK_CLIENT_SECRET")
                .map_err(|_| anyhow::anyhow!("KEYCLOAK_CLIENT_SECRET not set"))?;

            let kc = governance_core::keycloak::KeycloakClient::connect(
                &keycloak_conf.url,
                &keycloak_conf.realm,
                &client_id,
                &client_secret,
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            let forgejo_url = data
                .org
                .org
                .forgejo
                .as_ref()
                .map(|f| f.url().to_string())
                .unwrap_or_default();

            let result = kc
                .lookup_identity_links(codeberg_user, &forgejo_url)
                .map_err(|e| anyhow::anyhow!(e))?;

            println!("{}", serde_json::to_string(&result)?);
        }
        Command::CheckPr {
            author,
            base_ref,
            changed_files,
        } => {
            let data = GovernanceData::load(&cli.data_dir)?;
            let result =
                governance_core::check_pr::check_pr(&data, &author, &base_ref, &changed_files);
            if result.passed {
                eprintln!("PR check passed");
            } else {
                for issue in &result.issues {
                    eprintln!("denied: {issue}");
                }
                process::exit(1);
            }
        }
        Command::SlackInvite { channel, user } => {
            let token =
                std::env::var("SLACK_TOKEN").map_err(|_| anyhow::anyhow!("SLACK_TOKEN not set"))?;
            // Join the channel first so the bot can invite others
            let _ = ureq::post("https://slack.com/api/conversations.join")
                .header("Authorization", &format!("Bearer {token}"))
                .send_json(&serde_json::json!({ "channel": channel }));
            let resp = ureq::post("https://slack.com/api/conversations.invite")
                .header("Authorization", &format!("Bearer {token}"))
                .send_json(&serde_json::json!({
                    "channel": channel,
                    "users": user,
                }));
            match resp {
                Ok(mut r) => {
                    let body: serde_json::Value = r.body_mut().read_json()?;
                    if body["ok"].as_bool() != Some(true) {
                        let err = body["error"].as_str().unwrap_or("unknown");
                        if err != "already_in_channel" {
                            anyhow::bail!("slack invite failed: {err}");
                        }
                    }
                }
                Err(e) => anyhow::bail!("slack invite request failed: {e}"),
            }
        }
        Command::SlackKick { channel, user } => {
            let token =
                std::env::var("SLACK_TOKEN").map_err(|_| anyhow::anyhow!("SLACK_TOKEN not set"))?;
            let resp = ureq::post("https://slack.com/api/conversations.kick")
                .header("Authorization", &format!("Bearer {token}"))
                .send_json(&serde_json::json!({
                    "channel": channel,
                    "user": user,
                }));
            match resp {
                Ok(mut r) => {
                    let body: serde_json::Value = r.body_mut().read_json()?;
                    if body["ok"].as_bool() != Some(true) {
                        let err = body["error"].as_str().unwrap_or("unknown");
                        if err != "not_in_channel" {
                            anyhow::bail!("slack kick failed: {err}");
                        }
                    }
                }
                Err(e) => anyhow::bail!("slack kick request failed: {e}"),
            }
        }
    }

    Ok(())
}
