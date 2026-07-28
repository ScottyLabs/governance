use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use governance_core::loader::GovernanceData;
use governance_core::validator;
use governance_tfgen::axes;
use governance_tfgen::codeowners;
use governance_tfgen::generators::{
    cdn, discord, forgejo, github, google_groups, identities, keycloak, litellm, matrix_bridges,
    openbao, posthog, sentry, slack, vaultwarden,
};
use governance_tfgen::projects;
use governance_tfgen::tf_json::TfJsonFile;

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
    GenerateBridgeIdentityMap {
        #[arg(
            long,
            default_value = "../../infrastructure/services/matrix/bridge-identity-map.json"
        )]
        output: PathBuf,
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
    SlackJoin {
        #[arg(long)]
        channel: String,
    },
    SlackLeave {
        #[arg(long)]
        channel: String,
    },
    Projects,
    PosthogInvite {
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Command::Validate => cmd_validate(&cli.data_dir),
        Command::Generate { output_dir } => cmd_generate(&cli.data_dir, &output_dir),
        Command::Schema { output_dir } => cmd_schema(&output_dir),
        Command::GenerateBridgeIdentityMap { output } => {
            cmd_bridge_identity_map(&cli.data_dir, &output)
        }
        Command::ResolveIdentity => cmd_resolve_identity(&cli.data_dir),
        Command::Projects => cmd_projects(&cli.data_dir),
        Command::PosthogInvite { dry_run } => cmd_posthog_invite(&cli.data_dir, dry_run),
        Command::SlackInvite { channel, user } => cmd_slack_invite(&channel, &user),
        Command::SlackKick { channel, user } => cmd_slack_kick(&channel, &user),
        Command::SlackJoin { channel } => cmd_slack_join(&channel),
        Command::SlackLeave { channel } => cmd_slack_leave(&channel),
    }
}

fn cmd_validate(data_dir: &Path) -> anyhow::Result<()> {
    let data = GovernanceData::load(data_dir)?;
    let errors = validator::validate(&data);
    if errors.is_empty() {
        eprintln!("validation passed");
    } else {
        for err in &errors {
            eprintln!("error: {err}");
        }
        process::exit(1);
    }
    Ok(())
}

fn cmd_generate(data_dir: &Path, output_dir: &Path) -> anyhow::Result<()> {
    let data = GovernanceData::load(data_dir)?;
    std::fs::create_dir_all(output_dir)?;

    std::fs::create_dir_all(".forgejo")?;
    std::fs::write(
        ".forgejo/CODEOWNERS",
        codeowners::generate_codeowners(&data),
    )?;
    eprintln!("wrote .forgejo/CODEOWNERS");

    let shared = |name: &str| {
        let mut tf = TfJsonFile::default();
        axes::add_shared_data(&mut tf, name);
        tf
    };

    let dir = output_dir.join("keycloak");
    std::fs::create_dir_all(&dir)?;
    axes::framework("keycloak", &["keycloak", "vault", "external"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    shared("keycloak_realm").write_to(&dir.join("data.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    keycloak::generate_groups(&data).write_to(&dir.join("keycloak_groups.tf.json"))?;
    keycloak::generate_group_memberships(&data)
        .write_to(&dir.join("keycloak_memberships.tf.json"))?;
    keycloak::generate_clients(&data).write_to(&dir.join("keycloak_clients.tf.json"))?;

    let dir = output_dir.join("git");
    std::fs::create_dir_all(&dir)?;
    axes::framework(
        "git",
        &["forgejo", "github", "restapi", "random", "external"],
        &["kennel_webhook_url", "docs_webhook_url"],
    )
    .write_to(&dir.join("providers.tf.json"))?;
    shared("forgejo_organization").write_to(&dir.join("data.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    forgejo::generate_repos(&data).write_to(&dir.join("forgejo_repos.tf.json"))?;
    forgejo::generate_teams(&data).write_to(&dir.join("forgejo_teams.tf.json"))?;
    forgejo::generate_team_memberships(&data).write_to(&dir.join("forgejo_memberships.tf.json"))?;
    forgejo::generate_push_mirrors(&data).write_to(&dir.join("forgejo_push_mirrors.tf.json"))?;
    forgejo::generate_kennel_webhooks(&data)
        .write_to(&dir.join("forgejo_kennel_webhooks.tf.json"))?;
    forgejo::generate_docs_webhooks(&data).write_to(&dir.join("forgejo_docs_webhooks.tf.json"))?;
    forgejo::generate_team_repos(&data).write_to(&dir.join("forgejo_team_repos.tf.json"))?;
    github::generate_repos(&data).write_to(&dir.join("github_repos.tf.json"))?;
    github::generate_teams(&data).write_to(&dir.join("github_teams.tf.json"))?;
    github::generate_team_memberships(&data).write_to(&dir.join("github_memberships.tf.json"))?;

    let dir = output_dir.join("openbao");
    std::fs::create_dir_all(&dir)?;
    axes::framework("openbao", &["vault"], &[]).write_to(&dir.join("providers.tf.json"))?;
    shared("vault_auth_backend_oidc").write_to(&dir.join("data.tf.json"))?;
    openbao::generate_project_policies(&data).write_to(&dir.join("openbao.tf.json"))?;

    let dir = output_dir.join("google_groups");
    std::fs::create_dir_all(&dir)?;
    axes::framework("google_groups", &["google", "external"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    google_groups::generate(&data).write_to(&dir.join("google_groups.tf.json"))?;

    let dir = output_dir.join("discord");
    std::fs::create_dir_all(&dir)?;
    axes::framework("discord", &["discord", "external"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    discord::generate(&data).write_to(&dir.join("discord.tf.json"))?;

    let dir = output_dir.join("slack");
    std::fs::create_dir_all(&dir)?;
    axes::framework("slack", &["null", "external"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    slack::generate(&data).write_to(&dir.join("slack.tf.json"))?;

    let dir = output_dir.join("posthog");
    std::fs::create_dir_all(&dir)?;
    axes::framework("posthog", &["posthog", "vault"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    posthog::generate(&data).write_to(&dir.join("posthog.tf.json"))?;

    let dir = output_dir.join("sentry");
    std::fs::create_dir_all(&dir)?;
    axes::framework("sentry", &["sentry", "vault"], &["sentry_organization"])
        .write_to(&dir.join("providers.tf.json"))?;
    sentry::generate(&data).write_to(&dir.join("sentry.tf.json"))?;

    let dir = output_dir.join("litellm");
    std::fs::create_dir_all(&dir)?;
    axes::framework("litellm", &["vault", "litellm"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    shared("litellm_master_key").write_to(&dir.join("data.tf.json"))?;
    litellm::generate(&data).write_to(&dir.join("litellm.tf.json"))?;

    let dir = output_dir.join("vaultwarden");
    std::fs::create_dir_all(&dir)?;
    axes::framework("vaultwarden", &["bitwarden", "external"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    identities::generate_identity_data_sources(&data).write_to(&dir.join("identities.tf.json"))?;
    vaultwarden::generate(&data).write_to(&dir.join("vaultwarden.tf.json"))?;

    let dir = output_dir.join("cdn");
    std::fs::create_dir_all(&dir)?;
    axes::framework(
        "cdn",
        &["garage", "vault"],
        &["cdn_base_url", "garage_s3_endpoint"],
    )
    .write_to(&dir.join("providers.tf.json"))?;
    cdn::generate(&data).write_to(&dir.join("cdn.tf.json"))?;

    let dir = output_dir.join("matrix_bridges");
    std::fs::create_dir_all(&dir)?;
    axes::framework("matrix_bridges", &["synapse", "null"], &[])
        .write_to(&dir.join("providers.tf.json"))?;
    matrix_bridges::generate(&data).write_to(&dir.join("matrix_bridges.tf.json"))?;

    const AXES: &[&str] = &[
        "keycloak",
        "git",
        "openbao",
        "google_groups",
        "discord",
        "slack",
        "posthog",
        "sentry",
        "litellm",
        "vaultwarden",
        "cdn",
        "matrix_bridges",
    ];
    let repo_root = output_dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::write(repo_root.join("atlantis.yaml"), axes::atlantis_yaml(AXES))?;
    eprintln!("wrote {}/atlantis.yaml", repo_root.display());

    eprintln!("wrote {}", output_dir.display());
    Ok(())
}

fn cmd_schema(output_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)?;

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
    Ok(())
}

fn cmd_bridge_identity_map(data_dir: &Path, output: &Path) -> anyhow::Result<()> {
    let data = GovernanceData::load(data_dir)?;
    governance_core::bridge_identity::write_bridge_identity_map(&data, output)
        .map_err(|e| anyhow::anyhow!(e))?;
    eprintln!("wrote {}", output.display());
    Ok(())
}

fn cmd_resolve_identity(data_dir: &Path) -> anyhow::Result<()> {
    let query: serde_json::Value = serde_json::from_reader(std::io::stdin())?;
    let codeberg_user = query["codeberg_user"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing codeberg_user in query"))?;

    let data = GovernanceData::load(data_dir)?;
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
    Ok(())
}

fn cmd_projects(data_dir: &Path) -> anyhow::Result<()> {
    let data = GovernanceData::load(data_dir)?;
    print!("{}", projects::generate_projects(&data));
    Ok(())
}

fn cmd_posthog_invite(data_dir: &Path, dry_run: bool) -> anyhow::Result<()> {
    let data = GovernanceData::load(data_dir)?;
    governance_core::posthog::reconcile_invites(&data, dry_run).map_err(|e| anyhow::anyhow!(e))
}

fn cmd_slack_invite(channel: &str, user: &str) -> anyhow::Result<()> {
    governance_core::slack::invite(channel, user).map_err(|e| anyhow::anyhow!(e))
}

fn cmd_slack_kick(channel: &str, user: &str) -> anyhow::Result<()> {
    governance_core::slack::kick(channel, user).map_err(|e| anyhow::anyhow!(e))
}

fn cmd_slack_join(channel: &str) -> anyhow::Result<()> {
    governance_core::slack::join(channel).map_err(|e| anyhow::anyhow!(e))
}

fn cmd_slack_leave(channel: &str) -> anyhow::Result<()> {
    governance_core::slack::leave(channel).map_err(|e| anyhow::anyhow!(e))
}
