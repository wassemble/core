mod dependencies;
mod node_graph;

use context::Context;
use tokio_stream::Stream;

use std::collections::HashMap;

use dependencies::Dependencies;
use node_graph::NodeGraph;
use petgraph::Graph;
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Func, Linker, ResourceTable, Val},
};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};
use workflow::{ComponentName, FunctionName, InputName, Node, NodeId, Workflow};

pub struct Runtime<'a> {
    pub context: &'a mut Context,
    pub dependencies: Dependencies,
    pub node_graph: NodeGraph,
}

impl<'a> Runtime<'a> {
    pub async fn new(context: &'a mut Context, workflow: &Workflow) -> Result<Self, Error> {
        let dependencies = Dependencies::new(&context.engine, workflow).await?;
        let node_graph = NodeGraph::new(workflow)?;
        Ok(Self {
            dependencies,
            context,
            node_graph,
        })
    }

    pub fn run(&mut self, workflow: &Workflow) -> impl Stream<Item = (NodeId, Result<Val, Error>)> {
        // Could possibly be Vec<Val> because wasm functions can return multiple values
        // But for now, we only support one output value
        let mut values: HashMap<NodeId, Val> = HashMap::new();

        async_stream::stream! {
            for (node_id, node, edges) in self.node_graph.iter() {
                let result: Result<Val, Error> = async {
                    let inputs = edges.into_iter().map(|(input_name, node_id)| {
                        (input_name.clone(), values.get(node_id).unwrap().clone())
                    }).collect();
                    let value = self.run_node(node, &inputs).await?;
                    values.insert(node_id.clone(), value.clone());
                    Ok(value)
                }.await;
                yield (node_id.clone(), result);
            }
        }
    }

    async fn run_node(
        &mut self,
        node: &Node,
        inputs: &HashMap<InputName, Val>,
    ) -> Result<Val, Error> {
        let func = self.get_func(node).await?;
        let params = func
            .params(&mut self.context.store)
            .iter()
            .map(|(name, ty)| -> Result<Val, Error> {
                let input_name = InputName(name.to_string());
                match inputs.get(&input_name) {
                    Some(val) => Ok(val.clone()),
                    None => {
                        if let Some(input) = node.with.get(&input_name) {
                            Ok(Val::from_wave(ty, input)?)
                        } else {
                            Err(Error::MissingParameter(
                                node.run.clone(),
                                InputName(name.to_string()),
                            ))
                        }
                    }
                }
            })
            .collect::<Result<Vec<Val>, Error>>()?;

        let mut output = vec![Val::S32(0)];
        func.call_async(&mut self.context.store, &params, &mut output)
            .await?;

        Ok(output[0].clone())
    }

    async fn get_func(&mut self, node: &Node) -> Result<Func, Error> {
        let component = self.dependencies.get_component(&node.r#use)?;

        // Get the instance first
        let instance = self
            .context
            .linker
            .instantiate_async(&mut self.context.store, component)
            .await?;

        // Then get the function
        let func = instance
            .get_func(&mut self.context.store, &node.run.0)
            .ok_or_else(|| {
                Error::FunctionNotFound(node.run.clone(), self.get_available_exports(&component))
            })?;

        Ok(func)
    }

    fn get_available_exports(&self, component: &Component) -> Vec<String> {
        component
            .component_type()
            .exports(&self.context.engine)
            .map(|(name, _)| name.to_string())
            .collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Dependencies error: {0}")]
    Dependencies(#[from] dependencies::Error),
    #[error("Function {0} not found. Available exports: {1:?}")]
    FunctionNotFound(FunctionName, Vec<String>),
    #[error("Missing parameter: {0} for node {1}")]
    MissingParameter(FunctionName, InputName),
    #[error("Node graph error: {0}")]
    NodeGraph(#[from] node_graph::Error),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
    #[error("WasmWave error: {0}")]
    WasmWave(#[from] wasm_wave::parser::ParserError),
}
