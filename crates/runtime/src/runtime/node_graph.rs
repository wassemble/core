use petgraph::{Graph, acyclic::Acyclic, graph::NodeIndex, visit::EdgeRef};
use std::collections::HashMap;
use wasmtime::component::{Func, Val};
use workflow::{InputName, NodeId, Workflow};

use crate::context::Context;

use super::dependencies::Dependencies;

pub struct NodeGraph(Acyclic<Graph<Node, InputName>>);

struct Node {
    id: NodeId,
    func: Func,
    inputs: HashMap<InputName, Val>,
}

type Connections = HashMap<InputName, NodeId>;

impl NodeGraph {
    pub fn new(
        context: &mut Context,
        dependencies: &Dependencies,
        workflow: &Workflow,
    ) -> Result<Self, Error> {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        for (id, node) in &workflow.nodes {
            let node_index = graph.add_node((id.clone(), node.clone()));
            node_indices.insert(id, node_index);
        }

        for edge in &workflow.edges {
            if let (Some(source_idx), Some(target_idx)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                graph.add_edge(*source_idx, *target_idx, edge.input.clone());
            }
        }

        let acyclic =
            Acyclic::try_from_graph(graph).map_err(|cycle| Error::Cycle(cycle.node_id()))?;
        Ok(Self(acyclic))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Node, Connections)> + '_ {
        self.0.nodes_iter().map(|index| {
            let node = &self.0.node_weight(index).unwrap();
            let edges = self
                .0
                .edges_directed(index, petgraph::Incoming)
                .map(|edge| {
                    let input_name = edge.weight();
                    let node_id = &self.0.node_weight(edge.source()).unwrap().0;
                    (input_name, node_id)
                })
                .collect();
            (node_id, node, edges)
        })
    }

    pub fn node_weight(&self, index: NodeIndex) -> Option<&Node> {
        self.0.node_weight(index).map(|(_, node)| node)
    }

    pub fn edges_directed(
        &self,
        index: NodeIndex,
        direction: petgraph::Direction,
    ) -> impl Iterator<Item = petgraph::graph::EdgeReference<'_, InputName>> + '_ {
        self.0.edges_directed(index, direction)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cycle detected: {0:?}")]
    Cycle(NodeIndex),
}
