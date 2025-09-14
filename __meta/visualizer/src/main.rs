mod graph;

use askama::Template;
use governance::loader::{load_contributors, load_repos, load_teams};
use graph::build_graph_data;
use serde_json::Value;
use std::{error::Error, fs, path::Path, process};

#[derive(Template)]
#[template(path = "index.html")]
struct GovernanceTemplate {
    graph_data: Value,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Ensure this is being run from the workspace root
    if !Path::new("contributors").exists() {
        eprintln!("Please run this binary from the workspace root.");
        process::exit(1);
    }

    // Create output directory
    let dist_dir = Path::new("dist");
    fs::create_dir_all(dist_dir)?;

    // Load governance data
    let contributors = load_contributors()?;
    let teams = load_teams()?;
    let repos = load_repos()?;

    let graph_data = build_graph_data(contributors, teams, repos)?;

    // Render template
    let template = GovernanceTemplate { graph_data };
    let html = template.render()?;

    fs::write("dist/index.html", html)?;
    Ok(())
}
