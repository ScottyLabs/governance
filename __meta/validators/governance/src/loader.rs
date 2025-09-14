use crate::model::{Contributor, EntityKey, Repo, Team};
use anyhow::{Context, Result};
use glob::glob;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::{collections::HashMap, fs};

const CONTRIBUTORS_PATH: &str = "contributors/*.toml";
const TEAMS_PATH: &str = "teams/*.toml";
const REPOS_PATH: &str = "repos/*.toml";

pub fn load_from_dir<T: DeserializeOwned + Debug>(
    path_glob: &str,
    item_name: &str,
) -> Result<HashMap<EntityKey, T>> {
    let mut map = HashMap::new();
    for entry in glob(path_glob)? {
        let path = entry?;
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {} file: {}", item_name, path.display()))?;
        let item: T = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {} file: {}", item_name, path.display()))?;

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

pub fn load_repos() -> Result<HashMap<EntityKey, Repo>> {
    load_from_dir(REPOS_PATH, "repo")
}
