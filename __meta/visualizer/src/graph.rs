use governance::model::{Contributor, EntityKey, Team};
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

            let members = team
                .leads
                .iter()
                .chain(team.devs.iter())
                .collect::<Vec<_>>();

            for member_id in &members {
                let target_id = EntityKey {
                    kind: "contributor".to_string(),
                    name: member_id.to_string(),
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
