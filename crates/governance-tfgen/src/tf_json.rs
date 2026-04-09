use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

#[derive(Default, Serialize)]
pub struct TfJsonFile {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub locals: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub resource: BTreeMap<String, BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub data: BTreeMap<String, BTreeMap<String, Value>>,
}

impl TfJsonFile {
    pub fn add_local(&mut self, name: &str, value: Value) {
        self.locals.insert(name.to_string(), value);
    }

    pub fn add_resource(&mut self, resource_type: &str, name: &str, body: Value) {
        self.resource
            .entry(resource_type.to_string())
            .or_default()
            .insert(name.to_string(), body);
    }

    pub fn add_data(&mut self, data_type: &str, name: &str, body: Value) {
        self.data
            .entry(data_type.to_string())
            .or_default()
            .insert(name.to_string(), body);
    }

    pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).expect("TfJsonFile is always serializable");
        std::fs::write(path, json)
    }
}
