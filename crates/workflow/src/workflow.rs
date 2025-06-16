mod edge;
mod node;
mod types;
use std::{collections::HashMap, path::PathBuf};

pub use edge::*;
use file_source::FileSource;
pub use node::*;
use serde::{Deserialize, Serialize};
pub use types::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Workflow {
    pub dependencies: HashMap<ComponentName, FileSource>,
    pub edges: Vec<Edge>,
    pub nodes: HashMap<NodeId, Node>,
}

impl Workflow {
    pub fn load(path: &PathBuf) -> Result<Self, Error> {
        let source = std::fs::read_to_string(path)?;
        let is_json = path.extension().is_some_and(|ext| ext == "json");
        let workflow: Workflow = match is_json {
            true => serde_json::from_str(&source)?,
            false => serde_yaml::from_str(&source)?,
        };
        Ok(workflow)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_workflow() {
        let json = include_str!("../../../examples/hello-world.json");
        let _: Workflow = serde_json::from_str(json).unwrap();
    }
}
