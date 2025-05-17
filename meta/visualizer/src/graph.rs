use governance::model::{Contributor, EntityKey, Repo, Team};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

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
        inner: Repo,
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

struct GraphBuilder<'a> {
    contributors: &'a HashMap<EntityKey, Contributor>,
    teams: &'a HashMap<EntityKey, Team>,
    repos: &'a HashMap<EntityKey, Repo>,
}

impl<'a> GraphBuilder<'a> {
    fn new(
        contributors: &'a HashMap<EntityKey, Contributor>,
        teams: &'a HashMap<EntityKey, Team>,
        repos: &'a HashMap<EntityKey, Repo>,
    ) -> Self {
        Self {
            contributors,
            teams,
            repos,
        }
    }

    fn build_contributors_teams_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        // Add contributor nodes
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

            for member_id in &team.members {
                let target_id = EntityKey {
                    kind: "contributor".to_string(),
                    name: member_id.clone(),
                };

                links.push(GraphLink {
                    source: id.scoped_id(),
                    target: target_id.scoped_id(),
                    link_type: "team-member".to_string(),
                });
            }
        }

        GraphData { nodes, links }
    }

    fn build_teams_repos_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        // Add team nodes
        for (id, team) in self.teams {
            nodes.push(GraphNode::Team {
                id: id.scoped_id(),
                inner: team.clone(),
            });
        }

        // Add repo nodes
        for (id, repo) in self.repos {
            nodes.push(GraphNode::Repo {
                id: id.scoped_id(),
                inner: repo.clone(),
            });
        }

        // Add team-repo links
        for (team_id, team) in self.teams {
            for repo_id in &team.repos {
                let target_id = EntityKey {
                    kind: "repo".to_string(),
                    name: repo_id.clone(),
                };

                links.push(GraphLink {
                    source: team_id.scoped_id(),
                    target: target_id.scoped_id(),
                    link_type: "team-repo".to_string(),
                });
            }
        }

        GraphData { nodes, links }
    }

    fn build_contributors_repos_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        // Map to track which repos each contributor is connected to (via teams)
        let mut contributor_to_repos: HashMap<String, HashSet<String>> = HashMap::new();

        // Build the contributor-team-repo connection map
        for team in self.teams.values() {
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
        for (id, contributor) in self.contributors {
            nodes.push(GraphNode::Contributor {
                id: id.scoped_id(),
                inner: contributor.clone(),
            });
        }

        // Add repo nodes
        for (id, repo) in self.repos {
            nodes.push(GraphNode::Repo {
                id: id.scoped_id(),
                inner: repo.clone(),
            });
        }

        // Add contributor-repo links
        for (contributor_id, repo_ids) in contributor_to_repos {
            for repo_id in repo_ids {
                let source_id = EntityKey {
                    kind: "contributor".to_string(),
                    name: contributor_id.clone(),
                };

                let target_id = EntityKey {
                    kind: "repo".to_string(),
                    name: repo_id.clone(),
                };

                links.push(GraphLink {
                    source: source_id.scoped_id(),
                    target: target_id.scoped_id(),
                    link_type: "contributor-repo".to_string(),
                });
            }
        }

        GraphData { nodes, links }
    }
}

pub fn build_graph_data(
    contributors: HashMap<EntityKey, Contributor>,
    teams: HashMap<EntityKey, Team>,
    repos: HashMap<EntityKey, Repo>,
) -> Result<Value, Box<dyn Error>> {
    // Build filtered views
    let builder = GraphBuilder::new(&contributors, &teams, &repos);

    Ok(json!({
        "default": builder.build_contributors_teams_graph(),
        "teamsRepos": builder.build_teams_repos_graph(),
        "contributorsRepos": builder.build_contributors_repos_graph(),
    }))
}
