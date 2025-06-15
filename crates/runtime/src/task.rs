use petgraph::Directed;
use petgraph::graph::Edges;
use petgraph::{Graph, acyclic::Acyclic, graph::NodeIndex, visit::EdgeRef};
use std::collections::HashMap;
use tokio_stream::Stream;
use wasmtime::component::{Component, Func};
use wasmtime::component::{Instance, Val};
use wasmtime::{
    Config, Engine, Result, Store,
    component::{Linker, ResourceTable},
};
use wasmtime_wasi_http::WasiHttpCtx;
use workflow::NodeId;
use workflow::{ComponentName, InputName, Workflow};

use crate::prototype::{NodeType, Prototype};
use crate::runtime::Runtime;
use crate::state::State;

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
struct Task {
    graph: Acyclic<Graph<NodeType, InputName>>,
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
                .instantiate_async(&mut store, &component)
                .await?;
            instances.insert(component_name.clone(), instance);
        }

        Ok(Self {
            graph: prototype.graph.clone(),
            instances,
            store,
        })
    }

    pub fn run(&mut self) -> impl Stream<Item = (NodeId, Result<Val, Error>)> {
        async_stream::stream! {
            for (node, inputs) in self.iter() {
                if let NodeType::Function {
                    component_name,
                    index,
                    node_id,
                    val,
                } = node
                {
                    // If the node has no value, it means it's a function that needs to be executed
                    if let None = val {
                        // We compute the params from the edges
                        // We can unwrap here because everything should have been checked at prototype creation
                        let instance = self.instances.get(component_name).unwrap();
                        let func = instance.get_func(&mut self.store, *index).unwrap();
                        let mut params = Vec::new();
                        for (name, _) in func.params(&self.store) {
                            match inputs.get(&InputName(name.to_string())).unwrap() {
                                NodeType::Value(val) => params.push(val.clone()),
                                NodeType::Function {
                                    val,
                                    ..
                                } => params.push(val.unwrap().clone()),
                            }
                        }

                        // We execute the function and yield the result
                        let mut outputs = Vec::new();
                        if let Err(e) = func.call_async(&mut self.store, &params, &mut outputs).await {
                            yield (node_id.clone(), Err(Error::Wasmtime(e)));
                        }
                        let output = outputs[0].clone();
                        *val = Some(output.clone());
                        yield (node_id.clone(), Ok(output));
                    }
                }
            }
        }
    }

    fn iter(
        &mut self,
    ) -> impl Iterator<Item = (&mut NodeType, HashMap<InputName, &NodeType>)> + '_ {
        self.graph.nodes_iter().map(|index| {
            let leaf = self.graph.node_weight_mut(index).unwrap();
            let edges = self.graph.edges_directed(index, petgraph::Incoming);
            let inputs = edges
                .map(|edge| {
                    let input_name = edge.weight().clone();
                    let source = edge.source();
                    let input_val = self.graph.node_weight(source).unwrap();
                    (input_name, input_val)
                })
                .collect();
            (leaf, inputs)
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
