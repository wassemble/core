use wasmtime::{
    Config, Engine, Store,
    component::{Component, Func, Linker, ResourceTable},
};
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

pub struct Context {
    pub engine: Engine,
    pub linker: Linker<State>,
    pub store: Store<State>,
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;

        let mut linker = Linker::<State>::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

        let mut builder = WasiCtxBuilder::new();

        let store = Store::new(
            &engine,
            State {
                ctx: builder.build(),
                table: ResourceTable::new(),
                http: WasiHttpCtx::new(),
            },
        );

        Ok(Self {
            engine,
            linker,
            store,
        })
    }

    pub async fn get_func(&mut self, component: &Component, name: &str) -> Result<Func, Error> {
        let instance = self
            .linker
            .instantiate_async(&mut self.store, component)
            .await?;
        instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| Error::FunctionNotFound(name.to_string()))
    }
}

pub struct State {
    ctx: WasiCtx,
    table: ResourceTable,
    http: WasiHttpCtx,
}

impl IoView for State {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for State {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}
