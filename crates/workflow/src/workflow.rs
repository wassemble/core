mod edge;
mod node;
mod types;
pub use edge::*;
pub use node::*;
pub use types::*;

use std::collections::HashMap;

use file_source::FileSource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Workflow {
    pub dependencies: HashMap<ComponentName, FileSource>,
    pub edges: Vec<Edge>,
    pub nodes: HashMap<NodeId, Node>,
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
