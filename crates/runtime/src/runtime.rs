pub use wasmtime::Error;
use wasmtime::{Config, Engine, component::Linker};

use crate::state::State;

/// The `Runtime` owns the global execution context for workflows.
///
/// It holds:
/// - A single `Engine`, which caches compiled modules and components.
/// - A shared `Linker`, which registers host functions, capabilities, and
///   shared components available to all workflows.
///
/// The `Runtime` can compile multiple `Prototype` instances (static workflows)
/// and spawn multiple independent `Task` executions from them.
pub struct Runtime {
    pub engine: Engine,
    pub linker: Linker<State>,
}

impl Runtime {
    pub fn new() -> Result<Self, Error> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::<State>::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
        Ok(Self { engine, linker })
    }
}
