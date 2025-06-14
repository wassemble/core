mod to_json_schema;

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use wasmtime::component::types::ComponentItem;

use crate::{
    engine::{function_name::FunctionName, source::Source, Engine},
    metadata::to_json_schema::ToJsonSchema,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Metadata {
    pub sources: HashMap<Source, HashMap<FunctionName, Func>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Func {
    pub inputs: HashMap<InputName, Input>,
    pub name: FunctionName,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InputName(pub String);

impl fmt::Display for InputName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Input {
    pub name: InputName,
    pub schema: String,
}

impl Metadata {
    pub async fn new(engine: &mut Engine, sources: Vec<Source>) -> Self {
        let mut metadata: Metadata = Default::default();
        engine.load_components(&sources).await.unwrap();
        for source in sources {
            let component = engine.get_component(&source).unwrap();
            let mut functions = HashMap::new();
            for (func_name, func) in component.component_type().exports(&engine.context.engine) {
                println!("func: {func:?}");
                if let ComponentItem::ComponentFunc(func) = func {
                    let function_name = FunctionName(func_name.to_string());
                    let mut function = Func {
                        name: function_name.clone(),
                        inputs: HashMap::new(),
                    };
                    for (name, ty) in func.params() {
                        let input_name = InputName(name.to_string());
                        function.inputs.insert(
                            input_name.clone(),
                            Input {
                                name: input_name.clone(),
                                schema: serde_json::to_string(&ty.to_json_schema()).unwrap(),
                            },
                        );
                    }
                    functions.insert(function_name, function);
                }
            }
            metadata.sources.insert(source, functions);
        }
        metadata
    }
}
