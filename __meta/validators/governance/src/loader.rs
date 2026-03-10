use crate::model::{Contributor, EntityKey, HasKeyOrder, Team};
use anyhow::{Context, Result};
use glob::glob;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;
use std::{collections::HashMap, fs};
use toml::from_str;

const CONTRIBUTORS_PATH: &str = "contributors/*.toml";
const TEAMS_PATH: &str = "teams/*.toml";

pub fn load_from_dir<T: DeserializeOwned + Debug + HasKeyOrder>(
    path_glob: &str,
    item_name: &str,
) -> Result<HashMap<EntityKey, T>> {
    let mut map = HashMap::new();
    for entry in glob(path_glob)? {
        let path = entry?;
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {} file: {}", item_name, path.display()))?;

        // Calculate the key order for the item.
        let mut item: T = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {} file: {}", item_name, path.display()))?;
        let index_map: IndexMap<String, Value> = from_str(&content)?; // Parse into IndexMap to get key order.
        let key_order: Vec<String> = index_map.keys().cloned().collect();
        item.set_key_order(key_order);

        let key = EntityKey {
            kind: item_name.to_string(),
            name: file_stem,
        };
        map.insert(key, item);
    }

    Ok(map)
}

pub fn load_contributors() -> Result<HashMap<EntityKey, Contributor>> {
    load_from_dir(CONTRIBUTORS_PATH, "contributor")
}

pub fn load_teams() -> Result<HashMap<EntityKey, Team>> {
    load_from_dir(TEAMS_PATH, "team")
}

#[derive(Debug, Deserialize)]
struct Schema {
    properties: IndexMap<String, Value>, // Use IndexMap to preserve order
}

pub fn load_schema_key_ordering(schema_path: &str) -> Result<Vec<String>> {
    let schema_str = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path))?;

    let data: Schema = serde_json::from_str(&schema_str)
        .with_context(|| format!("Failed to parse schema file: {}", schema_path))?;

    Ok(data.properties.keys().cloned().collect())
}
