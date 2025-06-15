use std::collections::HashMap;

use wasmtime::component::{Component, Instance};
use workflow::{ComponentName, Workflow};

use crate::context::Context;

pub struct Dependencies(HashMap<ComponentName, Instance>);

impl Dependencies {
    pub async fn new(context: &mut Context, workflow: &Workflow) -> Result<Self, Error> {
        let mut components = HashMap::new();
        for (component_name, file_source) in &workflow.dependencies {
            let bytes = file_source.load().await?;
            let component = Component::from_binary(&context.engine, &bytes)?;
            let instance = context
                .linker
                .instantiate_async(&mut context.store, &component)
                .await?;
            components.insert(component_name.clone(), instance);
        }
        Ok(Self(components))
    }

    pub fn get(&self, name: &ComponentName) -> Result<&Instance, Error> {
        self.0
            .get(name)
            .ok_or_else(|| Error::ComponentNotFound(name.clone()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Component not found: {0}")]
    ComponentNotFound(ComponentName),
    #[error("File source error: {0}")]
    FileSource(#[from] file_source::Error),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
