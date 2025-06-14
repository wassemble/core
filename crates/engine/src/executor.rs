use std::collections::HashMap;

use petgraph::{algo::toposort, graph::NodeIndex, visit::EdgeRef, Graph};
use tokio_stream::Stream;
use wasmtime::component::Val;

use crate::{
    engine::{function_name::FunctionName, Engine},
    metadata::InputName,
    workflow::{
        node::{Data, Node, NodeId},
        Workflow,
    },
};

pub struct Executor {
    graph: Graph<Node, usize>,
    nodes: Vec<NodeIndex>,
}

impl Executor {
    pub fn new(workflow: &Workflow) -> Result<Self, Error> {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        // Add nodes and store their indices
        for (id, node) in &workflow.nodes {
            let node_index = graph.add_node(node.clone());
            node_indices.insert(id.clone(), node_index);
        }

        // Add edges using the stored indices
        for edge in &workflow.edges {
            if let (Some(source_idx), Some(target_idx)) = (
                node_indices.get(&edge.source),
                node_indices.get(&edge.target),
            ) {
                graph.add_edge(*source_idx, *target_idx, 0);
            }
        }

        let nodes = toposort(&graph, None).map_err(|e| Error::CyclicGraph(e.node_id().index()))?;
        Ok(Self { graph, nodes })
    }

    pub fn run<'a>(
        self,
        engine: &'a mut Engine,
    ) -> impl Stream<Item = (NodeId, Result<Vec<String>, Error>)> + 'a {
        let mut node_outputs: HashMap<NodeId, Vec<Val>> = HashMap::new();

        async_stream::stream! {
            for node_idx in &self.nodes {
                let node = self.graph.node_weight(*node_idx).unwrap();
                let Data { function, source, interface, inputs } = &node.data;
                let result = async {
                    let values = {
                        let func = &engine.get_func(
                            source.clone(),
                            function.clone(),
                            interface.clone(),
                        ).await?;
                        let mut input_values: HashMap<usize, Val> = HashMap::new();

                        for edge in self.graph.edges_directed(*node_idx, petgraph::Incoming) {
                            let parent_idx = edge.source();
                            let parent = self.graph.node_weight(parent_idx).unwrap();
                            let edge_index = *edge.weight();
                            if let Some(parent_outputs) = node_outputs.get(&parent.id) {
                                if let Some(output) = parent_outputs.first() {
                                    input_values.insert(edge_index, output.clone());
                                }
                            }
                        }

                        let params = func
                            .params(&mut engine.context.store)
                            .iter()
                            .enumerate()
                            .map(|(i, (name, ty))| -> Result<Val, Error> {
                                match input_values.get(&i) {
                                    Some(val) => Ok(val.clone()),
                                    None => {
                                        if let Some(input) = inputs.get(name) {
                                            Ok(Val::from_wave(ty, input)?)
                                        } else {
                                            Err(Error::MissingParameter(function.clone(), InputName(name.to_string())))
                                        }
                                    }
                                }
                            })
                            .collect::<Result<Vec<Val>, Error>>()?;
                        let mut output = vec![Val::S32(0)];
                        func.call_async(&mut engine.context.store, &params, &mut output)
                            .await?;
                        output.to_vec()
                    };
                    tracing::info!("executed node {:?} with output {:?}", node, values.clone());
                    node_outputs.insert(node.id.clone(), values.clone());
                    Ok::<Vec<String>, Error>(values.iter().map(|v| v.to_wave().unwrap()).collect())
                }.await;
                yield (node.id.clone(), result);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cyclic graph detected at index {0}")]
    CyclicGraph(usize),
    #[error(transparent)]
    Engine(#[from] crate::engine::Error),
    #[error("Missing parameter: {0} for node {1}")]
    MissingParameter(FunctionName, InputName),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
    #[error("WasmWave error: {0}")]
    WasmWave(#[from] wasm_wave::parser::ParserError),
}
