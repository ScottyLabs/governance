use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use governance_core::loader::GovernanceData;
use governance_core::validator;
use governance_tfgen::codeowners;

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
        #[arg(long, default_value = "generated/tofu")]
        output_dir: PathBuf,
    },
    Schema {
        #[arg(long, default_value = "generated/schemas")]
        output_dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
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
            let errors = validator::validate(&data);
            if !errors.is_empty() {
                for err in &errors {
                    eprintln!("error: {err}");
                }
                process::exit(1);
            }

            std::fs::create_dir_all(&output_dir)?;

            std::fs::create_dir_all(".forgejo")?;
            let codeowners_content = codeowners::generate_codeowners(&data);
            std::fs::write(".forgejo/CODEOWNERS", codeowners_content)?;
            eprintln!("wrote .forgejo/CODEOWNERS");

            // TODO: generate .tf.json files
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
    }

    Ok(())
}
