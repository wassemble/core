pub mod edge;
pub mod node;

use std::collections::HashMap;

use edge::Edge;
use serde::{Deserialize, Serialize};

use self::node::{Node, NodeId};
use crate::engine::source::Source;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Workflow {
    pub edges: Vec<Edge>,
    pub nodes: HashMap<NodeId, Node>,
}

impl Workflow {
    pub fn sources(&self) -> Vec<Source> {
        let mut sources = vec![];
        for node in self.nodes.values() {
            sources.push(node.data.source.clone());
        }
        sources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_workflow() {
        let json = include_str!("../examples/hello-world.json");
        let _: Workflow = serde_json::from_str(json).unwrap();
    }
}
