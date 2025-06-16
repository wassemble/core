use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ComponentName, FunctionName, InputName};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    /// The function to run
    pub run: FunctionName,
    /// The component to use from the dependencies
    pub r#use: ComponentName,
    /// Optional manual inputs to the function (wasm_wave encoded)
    #[serde(default)]
    pub with: HashMap<InputName, String>,
}
