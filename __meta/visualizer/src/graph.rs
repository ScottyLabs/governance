use governance::model::{Contributor, EntityKey, Team};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, error::Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RepoInfo {
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "nodeType")]
enum GraphNode {
    Contributor {
        id: String,
        #[serde(flatten)]
        inner: Contributor,
    },
    Team {
        id: String,
        #[serde(flatten)]
        inner: Team,
    },
    Repo {
        id: String,
        #[serde(flatten)]
        inner: RepoInfo,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphLink {
    source: String,
    target: String,
    #[serde(rename = "linkType")]
    link_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

// Resolve the repository information (i.e. name and URL) from a reference string.
fn repo_info_from_ref(entry: &str) -> RepoInfo {
    let trimmed = entry.trim_end_matches('/').trim_end_matches(".git");
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        let name = trimmed.rsplit('/').next().unwrap_or(trimmed).to_string();
        RepoInfo {
            name,
            url: trimmed.to_string(),
        }
    } else if trimmed.contains('/') {
        RepoInfo {
            name: trimmed.to_string(),
            url: format!("https://github.com/{}", trimmed),
        }
    } else {
        RepoInfo {
            name: trimmed.to_string(),
            url: String::new(),
        }
    }
}

// Resolve the node ID for a repository from a reference string.
fn repo_node_id(entry: &str) -> String {
    let trimmed = entry.trim_end_matches('/').trim_end_matches(".git");
    format!("repo:{}", trimmed.replace('/', ":"))
}

struct GraphBuilder<'a> {
    contributors: &'a HashMap<EntityKey, Contributor>,
    teams: &'a HashMap<EntityKey, Team>,
}

impl<'a> GraphBuilder<'a> {
    fn new(
        contributors: &'a HashMap<EntityKey, Contributor>,
        teams: &'a HashMap<EntityKey, Team>,
    ) -> Self {
        Self {
            contributors,
            teams,
        }
    }

    fn build_contributors_teams_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut seen_repos = HashMap::<String, ()>::new();

        for (id, contributor) in self.contributors {
            nodes.push(GraphNode::Contributor {
                id: id.scoped_id(),
                inner: contributor.clone(),
            });
        }

        // Add team nodes and links
        for (id, team) in self.teams {
            nodes.push(GraphNode::Team {
                id: id.scoped_id(),
                inner: team.clone(),
            });

            for contributor_id in &team.contributors {
                let target_id = EntityKey {
                    kind: "contributor".to_string(),
                    name: contributor_id.to_string(),
                };

                links.push(GraphLink {
                    source: id.scoped_id(),
                    target: target_id.scoped_id(),
                    link_type: "team-member".to_string(),
                });
            }

            for entry in &team.repos {
                let node_id = repo_node_id(entry);

                if !seen_repos.contains_key(&node_id) {
                    seen_repos.insert(node_id.clone(), ());
                    nodes.push(GraphNode::Repo {
                        id: node_id.clone(),
                        inner: repo_info_from_ref(entry),
                    });
                }

                links.push(GraphLink {
                    source: id.scoped_id(),
                    target: node_id,
                    link_type: "team-repo".to_string(),
                });
            }
        }

        GraphData { nodes, links }
    }
}

pub fn build_graph_data(
    contributors: HashMap<EntityKey, Contributor>,
    teams: HashMap<EntityKey, Team>,
) -> Result<Value, Box<dyn Error>> {
    // Build filtered views
    let builder = GraphBuilder::new(&contributors, &teams);

    Ok(json!({
        "default": builder.build_contributors_teams_graph(),
    }))
}
