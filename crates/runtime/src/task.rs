use std::collections::HashMap;

use petgraph::{Graph, algo::toposort, graph::NodeIndex, visit::EdgeRef};
use tokio::sync::broadcast::{Receiver, Sender, channel};
use wasmtime::{
    Result, Store,
    component::{Instance, Val},
};
use workflow::{ComponentName, InputName, NodeId};

use crate::{
    prototype::{NodeType, Prototype},
    runtime::Runtime,
    state::State,
};

/// A `Task` represents a single, isolated execution of a workflow prototype.
///
/// It holds:
/// - A new `Store`, providing isolated memory, globals, tables, and state.
/// - An `Instance` of a `Prototype` within that store.
///
/// Each `Task` runs independently from others, even if derived from the same
/// `Prototype`. They may share capabilities via the `Program`'s `Linker`, but
/// do not share memory or instances directly. Any shared state must be managed
/// externally via host functions or global services.
pub struct Task {
    sender: Sender<(NodeId, Result<Val, String>)>,
    graph: Graph<NodeType, InputName>,
    instances: HashMap<ComponentName, Instance>,
    store: Store<State>,
}

impl Task {
    pub async fn new(runtime: &mut Runtime, prototype: &Prototype) -> Result<Self, Error> {
        let mut store = Store::new(&runtime.engine, State::new());

        let mut instances = HashMap::new();
        for (component_name, component) in &prototype.components {
            let instance = runtime
                .linker
                .instantiate_async(&mut store, component)
                .await?;
            instances.insert(component_name.clone(), instance);
        }

        let (sender, _) = channel(32);

        Ok(Self {
            sender,
            graph: prototype.graph.inner().clone(),
            instances,
            store,
        })
    }

    pub fn subscribe(&self) -> Receiver<(NodeId, Result<Val, String>)> {
        self.sender.subscribe()
    }

    pub async fn run(&mut self) {
        let nodes = self.prepare();
        for (node_index, inputs_index) in nodes {
            let node = self.graph.node_weight(node_index).unwrap();
            let mut val = None;
            if let NodeType::Function(function) = node {
                if function.val.is_none() {
                    let instance = self.instances.get(&function.component_name).unwrap();
                    let func = instance.get_func(&mut self.store, function.index).unwrap();

                    let mut params = Vec::new();
                    for (name, _) in func.params(&self.store) {
                        let input_index = *inputs_index.get(&InputName(name.to_string())).unwrap();
                        match self.graph.node_weight(input_index).unwrap() {
                            NodeType::Value(val) => params.push(val.clone()),
                            NodeType::Function(function) => {
                                params.push(function.val.clone().unwrap())
                            }
                        }
                    }

                    let mut outputs = Vec::new();
                    if let Err(e) = func
                        .call_async(&mut self.store, &params, &mut outputs)
                        .await
                    {
                        self.sender
                            .send((function.node_id.clone(), Err(e.to_string())))
                            .unwrap();
                    }
                    let output = outputs[0].clone();
                    val = Some(output.clone());
                    self.sender
                        .send((function.node_id.clone(), Ok(output.clone())))
                        .unwrap();
                }
            }

            if let Some(val) = val {
                self.graph.node_weight_mut(node_index).unwrap().set_val(val);
            }
        }
    }

    fn prepare(&self) -> Vec<(NodeIndex, HashMap<InputName, NodeIndex>)> {
        let mut nodes = Vec::new();
        for node_index in toposort(&self.graph, None).unwrap() {
            let edges = self.graph.edges_directed(node_index, petgraph::Incoming);
            let inputs = edges
                .map(|edge| (edge.weight().clone(), edge.source()))
                .collect();
            nodes.push((node_index, inputs));
        }
        nodes
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
