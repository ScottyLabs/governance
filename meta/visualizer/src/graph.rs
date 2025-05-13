use governance::model::{Contributor, Repo, Team};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct GraphNode {
    id: String,
    name: String,
    #[serde(rename = "nodeType")]
    node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    websites: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct GraphLink {
    source: String,
    target: String,
    #[serde(rename = "linkType")]
    link_type: String,
}

#[derive(Serialize, Deserialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

fn build_contributors_teams_graph(
    contributors: &HashMap<String, Contributor>,
    teams: &HashMap<String, Team>,
) -> Result<Value, Box<dyn Error>> {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Add contributor nodes
    for (id, contributor) in contributors {
        nodes.push(GraphNode {
            id: id.clone(),
            name: contributor.name.clone(),
            node_type: "Contributor".to_string(),
            github: Some(contributor.github.clone()),
            description: None,
            website: None,
            websites: None,
            members: None,
        });
    }

    // Add team nodes and team-member links
    for (id, team) in teams {
        nodes.push(GraphNode {
            id: id.clone(),
            name: team.name.clone(),
            node_type: "Team".to_string(),
            github: None,
            description: None,
            website: None,
            websites: None,
            members: Some(team.members.clone()),
        });

        for member_id in &team.members {
            links.push(GraphLink {
                source: id.clone(),
                target: member_id.clone(),
                link_type: "team-member".to_string(),
            });
        }
    }

    Ok(json!({
        "nodes": nodes,
        "links": links
    }))
}

fn build_teams_repos_graph(
    teams: &HashMap<String, Team>,
    repos: &HashMap<String, Repo>,
) -> Result<Value, Box<dyn Error>> {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Add team nodes
    for (id, team) in teams {
        nodes.push(GraphNode {
            id: id.clone(),
            name: team.name.clone(),
            node_type: "Team".to_string(),
            github: None,
            description: None,
            website: None,
            websites: None,
            members: Some(team.members.clone()),
        });
    }

    // Add repo nodes
    for (id, repo) in repos {
        nodes.push(GraphNode {
            id: id.clone(),
            name: repo.name.clone(),
            node_type: "Repo".to_string(),
            github: None,
            description: repo.description.clone(),
            website: repo.website.clone(),
            websites: repo.websites.clone(),
            members: None,
        });
    }

    // Add team-repo links
    for (team_id, team) in teams {
        for repo_id in &team.repos {
            links.push(GraphLink {
                source: team_id.clone(),
                target: repo_id.clone(),
                link_type: "team-repo".to_string(),
            });
        }
    }

    Ok(json!({
        "nodes": nodes,
        "links": links
    }))
}

fn build_contributors_repos_graph(
    contributors: &HashMap<String, Contributor>,
    teams: &HashMap<String, Team>,
    repos: &HashMap<String, Repo>,
) -> Result<Value, Box<dyn Error>> {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Map to track which repos each contributor is connected to (via teams)
    let mut contributor_to_repos: HashMap<String, HashSet<String>> = HashMap::new();

    // Build the contributor-team-repo connection map
    for team in teams.values() {
        for member_id in &team.members {
            for repo_id in &team.repos {
                contributor_to_repos
                    .entry(member_id.clone())
                    .or_default()
                    .insert(repo_id.clone());
            }
        }
    }

    // Add contributor nodes
    for (id, contributor) in contributors {
        nodes.push(GraphNode {
            id: id.clone(),
            name: contributor.name.clone(),
            node_type: "Contributor".to_string(),
            github: Some(contributor.github.clone()),
            description: None,
            website: None,
            websites: None,
            members: None,
        });
    }

    // Add repo nodes
    for (id, repo) in repos {
        nodes.push(GraphNode {
            id: id.clone(),
            name: repo.name.clone(),
            node_type: "Repo".to_string(),
            github: None,
            description: repo.description.clone(),
            website: repo.website.clone(),
            websites: repo.websites.clone(),
            members: None,
        });
    }

    // Add contributor-repo links
    for (contributor_id, repo_ids) in contributor_to_repos {
        for repo_id in repo_ids {
            links.push(GraphLink {
                source: contributor_id.clone(),
                target: repo_id.clone(),
                link_type: "contributor-repo".to_string(),
            });
        }
    }

    Ok(json!({
        "nodes": nodes,
        "links": links
    }))
}

pub fn build_graph_data(
    contributors: HashMap<String, Contributor>,
    teams: HashMap<String, Team>,
    repos: HashMap<String, Repo>,
) -> Result<Value, Box<dyn Error>> {
    // Build filtered views
    let default = build_contributors_teams_graph(&contributors, &teams)?;
    let teams_repos = build_teams_repos_graph(&teams, &repos)?;
    let contributors_repos = build_contributors_repos_graph(&contributors, &teams, &repos)?;

    Ok(json!({
        "default": default,
        "teamsRepos": teams_repos,
        "contributorsRepos": contributors_repos
    }))
}
