pub mod context;
pub mod function_name;
pub mod interface_name;
pub mod source;

use std::collections::HashMap;

use context::Context;
use function_name::FunctionName;
use interface_name::InterfaceName;
use source::Source;
use wasmtime::component::{Component, Func};

pub struct Engine {
    pub context: Context,
    pub store: HashMap<Source, Component>,
}

impl Engine {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            context: Context::new()?,
            store: HashMap::new(),
        })
    }

    pub async fn load_component(&mut self, source: &Source) -> Result<(), Error> {
        if self.store.get(source).cloned().is_none() {
            let component_bytes = source.clone().load().await?;
            let component = Component::new(&self.context.engine, &component_bytes)?;
            self.store.insert(source.clone(), component.clone());
        }
        Ok(())
    }

    pub async fn load_components(&mut self, sources: &[Source]) -> Result<(), Error> {
        for source in sources {
            self.load_component(source).await?;
        }
        Ok(())
    }

    pub fn get_component(&self, source: &Source) -> Result<&Component, Error> {
        self.store
            .get(source)
            .ok_or(Error::ComponentNotFound(source.clone()))
    }

    pub fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub async fn get_func(
        &mut self,
        source: Source,
        function_name: FunctionName,
        interface_name: Option<InterfaceName>,
    ) -> Result<Func, Error> {
        let component = self.get_component(&source)?.clone();
        let context = self.get_context_mut();
        let instance = context
            .linker
            .instantiate_async(&mut context.store, &component)
            .await?;
        Ok(match interface_name {
            None => instance
                .get_func(&mut self.context.store, &function_name.0)
                .ok_or(Error::FunctionNotFound(
                    function_name,
                    self.get_available_exports(&component),
                ))?,
            Some(interface_name) => {
                let interface = instance
                    .get_export(&mut self.context.store, None, &interface_name.0)
                    .ok_or_else(|| {
                        Error::InterfaceExportNotFound(
                            interface_name.clone(),
                            self.get_available_exports(&component),
                        )
                    })?;
                let export = instance
                    .get_export(&mut self.context.store, Some(&interface), &function_name.0)
                    .ok_or_else(|| {
                        Error::InterfaceFunctionNotFound(
                            function_name.clone(),
                            interface_name.clone(),
                            self.get_available_exports(&component),
                        )
                    })?;
                instance.get_func(&mut self.context.store, export).ok_or(
                    Error::FunctionNotFound(
                        function_name.clone(),
                        self.get_available_exports(&component),
                    ),
                )?
            }
        })
    }

    pub fn get_available_exports(&self, component: &Component) -> Vec<String> {
        component
            .component_type()
            .exports(&self.context.engine)
            .map(|(name, _)| name.to_string())
            .collect()
    }
}

type AvailableExports = Vec<String>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Component not found: {0}")]
    ComponentNotFound(Source),
    #[error("Function {0} not found. Available exports: {1:?}")]
    FunctionNotFound(FunctionName, AvailableExports),
    #[error("Interface export not found: {0}. Available exports: {1:?}")]
    InterfaceExportNotFound(InterfaceName, AvailableExports),
    #[error("Function {0} not found in interface {1}. Available exports: {2:?}")]
    InterfaceFunctionNotFound(FunctionName, InterfaceName, AvailableExports),
    #[error(transparent)]
    Source(#[from] source::Error),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
