mod checks;

use anyhow::{Result, anyhow};
use checks::{
    validate_cross_references, validate_file_names, validate_github_users, validate_slack_ids,
};
use colored::Colorize;
use dotenv::dotenv;
use governance::loader::{load_contributors, load_repos, load_teams};
use governance::model::{
    FileValidationMessages, ValidationError, ValidationReport, ValidationStatistics,
    ValidationWarning,
};
use log::error;
use reqwest::Client;
use std::{collections::HashMap, path::Path, process};

fn insert_error(files: &mut HashMap<String, FileValidationMessages>, error: ValidationError) {
    files
        .entry(error.file.clone())
        .or_default()
        .errors
        .push(error);
}

fn insert_warning(files: &mut HashMap<String, FileValidationMessages>, warning: ValidationWarning) {
    files
        .entry(warning.file.clone())
        .or_default()
        .warnings
        .push(warning);
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Ensure this is being run from the workspace root
    if !Path::new("contributors").exists() {
        error!("Please run this binary from the workspace root.");
        process::exit(1);
    }

    // Load data from files
    let contributors = load_contributors()?;
    let teams = load_teams()?;
    let repos = load_repos()?;

    let mut file_messages = contributors
        .keys()
        .map(|k| {
            (
                format!("contributors/{}.toml", k),
                FileValidationMessages::default(),
            )
        })
        .chain(teams.keys().map(|k| {
            (
                format!("teams/{}.toml", k),
                FileValidationMessages::default(),
            )
        }))
        .chain(repos.keys().map(|k| {
            (
                format!("repos/{}.toml", k),
                FileValidationMessages::default(),
            )
        }))
        .collect::<HashMap<_, _>>();

    // Validate file names
    for error in validate_file_names(&contributors, &teams, &repos) {
        insert_error(&mut file_messages, error);
    }

    // Validate cross-references
    for error in validate_cross_references(&contributors, &teams, &repos) {
        insert_error(&mut file_messages, error);
    }

    let client = Client::new();

    // Validate GitHub users
    let (errors, warnings) = validate_github_users(&contributors, &client).await;
    for error in errors {
        insert_error(&mut file_messages, error);
    }
    for warning in warnings {
        insert_warning(&mut file_messages, warning);
    }

    // Validate Slack IDs
    let (errors, warnings) = validate_slack_ids(&contributors, &teams, &client).await;
    for error in errors {
        insert_error(&mut file_messages, error);
    }
    for warning in warnings {
        insert_warning(&mut file_messages, warning);
    }

    // Generate validation report
    let total_errors = file_messages.values().map(|f| f.errors.len()).sum();
    let total_warnings = file_messages.values().map(|f| f.warnings.len()).sum();

    let (valid_files_count, invalid_files_count) =
        file_messages.values().fold((0, 0), |(valid, invalid), m| {
            if m.errors.is_empty() {
                (valid + 1, invalid)
            } else {
                (valid, invalid + 1)
            }
        });

    let stats = ValidationStatistics {
        contributors_count: contributors.len(),
        teams_count: teams.len(),
        repos_count: repos.len(),
        valid_files_count,
        invalid_files_count,
        total_errors,
        total_warnings,
    };

    let report = ValidationReport {
        valid: stats.invalid_files_count == 0,
        stats,
        files: file_messages,
    };

    // Display report
    println!("{}", "===== SUMMARY =====".blue().bold());
    println!("Contributors: {}", report.stats.contributors_count);
    println!("Teams: {}", report.stats.teams_count);
    println!("Repos: {}", report.stats.repos_count);
    println!("Valid files: {}", report.stats.valid_files_count);
    println!("Invalid files: {}", report.stats.invalid_files_count);
    println!("Total errors: {}", report.stats.total_errors);
    println!("Total warnings: {}", report.stats.total_warnings);

    if report.stats.total_errors > 0 {
        println!("\n{}", "===== ERRORS =====".red().bold());
        for (file, messages) in &report.files {
            if messages.errors.is_empty() {
                continue;
            }

            println!("{}", file);
            for error in &messages.errors {
                println!("  - {}", error.message);
            }
        }
    }

    if report.stats.total_warnings > 0 {
        println!("\n{}", "===== WARNINGS =====".yellow().bold());
        for (file, messages) in &report.files {
            if messages.warnings.is_empty() {
                continue;
            }

            println!("{}", file);
            for warning in &messages.warnings {
                println!("  - {}", warning.message);
            }
        }
    }

    if !report.valid {
        println!();
        return Err(anyhow!(
            "Validation failed with {} error(s) in {} file(s)",
            report.stats.total_errors.to_string().red().bold(),
            report.stats.invalid_files_count.to_string().red().bold()
        ));
    }

    println!("\n{}", "Validation passed!".green().bold());
    Ok(())
}
