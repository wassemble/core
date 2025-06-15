use std::collections::HashMap;

use petgraph::{
    Directed, Graph,
    acyclic::Acyclic,
    graph::{Edges, NodeIndex},
};
use wasmtime::component::{Component, ComponentExportIndex, Val, types::ComponentItem};
use workflow::{ComponentName, Edge, InputName, Node, NodeId, Workflow};

use crate::runtime::Runtime;

/// A `Prototype` represents a compiled, static workflow definition.
///
/// It holds:
/// - A list of compiled WebAssembly `Component`.
/// - A graph of nodes (functions and values) and edges (inputs) representing the workflow.
///
/// `Prototype` instances are created once and can be executed many times
/// by spawning new `Task` instances. Compilation is cached by the
/// `Program`'s `Engine`, making `Prototype` instantiation cheap.
pub struct Prototype {
    pub(crate) components: HashMap<ComponentName, Component>,
    pub(crate) graph: Acyclic<Graph<NodeType, InputName>>,
}

impl Prototype {
    pub async fn new(runtime: &mut Runtime, workflow: &Workflow) -> Result<Self, Error> {
        // Compiled components
        let mut components = HashMap::new();

        // Graph of nodes and edges
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        for (node_id, node) in &workflow.nodes {
            // First, we get the component or we load it
            let component = match components.get(&node.r#use) {
                Some(component) => component,
                None => {
                    let file_source = workflow
                        .dependencies
                        .get(&node.r#use)
                        .ok_or(Error::DependencyNotFound(node.r#use.clone()))?;
                    let bytes = file_source.load().await?;
                    let component = Component::from_binary(&runtime.engine, &bytes)?;
                    components.insert(node.r#use.clone(), component);
                    components.get(&node.r#use).unwrap()
                }
            };

            // Then, we lookup the corresponding function
            let (item, index) = component.export_index(None, &node.run).unwrap();

            if let ComponentItem::ComponentFunc(func) = item {
                // We add the node's function to the graph
                let node_index = graph.add_node(NodeType::Function(Function {
                    component_name: node.r#use.clone(),
                    index,
                    node_id: node_id.clone(),
                    val: None,
                }));
                node_indices.insert(node_id, node_index);

                // We add the node's inputs to the graph
                for (name, ty) in func.params() {
                    let input_name = InputName(name.to_string());
                    if let Some(value) = node.with.get(&input_name) {
                        let val = Val::from_wave(&ty, value)?;
                        let input_index = graph.add_node(NodeType::Value(val.clone()));
                        graph.add_edge(node_index, input_index, input_name.clone());
                    } else {
                        Err(Error::MissingInput(input_name.clone()))?;
                    }
                }
            } else {
                Err(Error::InvalidNode(node.clone()))?;
            }
        }

        // Finally, we connect the nodes' function to each other
        for edge in &workflow.edges {
            if let (Some(source_idx), Some(target_idx)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                graph.add_edge(*source_idx, *target_idx, edge.input.clone());
            } else {
                Err(Error::InvalidEdge(edge.clone()))?;
            }
        }

        let graph =
            Acyclic::try_from_graph(graph).map_err(|cycle| Error::Cycle(cycle.node_id()))?;

        Ok(Self { components, graph })
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Function(Function),
    Value(Val),
}

impl NodeType {
    pub fn set_val(&mut self, val: Val) {
        match self {
            NodeType::Function(function) => function.val = Some(val),
            NodeType::Value(val) => *val = val.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) component_name: ComponentName,
    pub(crate) index: ComponentExportIndex,
    pub(crate) node_id: NodeId,
    pub(crate) val: Option<Val>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cycle detected: {0:?}")]
    Cycle(NodeIndex),
    #[error("Dependency not found: {0:?}")]
    DependencyNotFound(ComponentName),
    #[error("File source error: {0}")]
    FileSource(#[from] file_source::Error),
    #[error("Invalid edge: {0:?}")]
    InvalidEdge(Edge),
    #[error("Invalid node: {0:?}")]
    InvalidNode(Node),
    #[error("Missing input: {0:?}")]
    MissingInput(InputName),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
