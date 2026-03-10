use governance::model::{Contributor, EntityKey, Repo, Team};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, error::Error};

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

/// Normalize a repo URL to "owner/repo" for matching legacy refs.
fn url_to_owner_repo(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    if let Some(path) = url.strip_prefix("https://github.com/") {
        return Some(path.to_string());
    }
    if let Some(path) = url.strip_prefix("https://codeberg.org/") {
        return Some(path.to_string());
    }
    None
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

    fn repo_node_id(slug: &str) -> String {
        format!("repo:{}", slug)
    }

    fn build_contributors_teams_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        let legacy_to_slug: HashMap<String, String> = self
            .repos
            .values()
            .filter_map(|r| url_to_owner_repo(&r.url).map(|legacy| (legacy, r.slug.clone())))
            .collect();

        for (id, contributor) in self.contributors {
            nodes.push(GraphNode::Contributor {
                id: id.scoped_id(),
                inner: contributor.clone(),
            });
        }

        for (id, repo) in self.repos {
            nodes.push(GraphNode::Repo {
                id: Self::repo_node_id(&id.name),
                inner: repo.clone(),
            });
        }

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

            for ref_ in &team.repos {
                let (repo_node_id, maybe_synthetic) = if ref_.contains('/') {
                    if let Some(slug) = legacy_to_slug.get(ref_.as_str()) {
                        (Self::repo_node_id(slug), None)
                    } else {
                        let node_id = format!("repo:{}", ref_.replace('/', ":"));
                        (
                            node_id.clone(),
                            Some(Repo {
                                slug: ref_.replace('/', ":"),
                                name: ref_.clone(),
                                description: None,
                                url: format!("https://github.com/{}", ref_),
                                key_order: vec![],
                            }),
                        )
                    }
                } else {
                    (Self::repo_node_id(ref_), None)
                };

                let has_node = if ref_.contains('/') {
                    true
                } else {
                    self.repos.contains_key(&EntityKey {
                        kind: "repo".to_string(),
                        name: ref_.clone(),
                    })
                };
                if has_node {
                    links.push(GraphLink {
                        source: id.scoped_id(),
                        target: repo_node_id.clone(),
                        link_type: "team-repo".to_string(),
                    });
                }
                if let Some(syn) = maybe_synthetic {
                    let node_id = format!("repo:{}", ref_.replace('/', ":"));
                    if !nodes.iter().any(|n| matches!(n, GraphNode::Repo { id: i, .. } if i == &node_id)) {
                        nodes.push(GraphNode::Repo {
                            id: node_id,
                            inner: syn,
                        });
                    }
                }
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
    let builder = GraphBuilder::new(&contributors, &teams, &repos);
    Ok(json!({
        "default": builder.build_contributors_teams_graph(),
    }))
}
