use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::engine::{function_name::FunctionName, interface_name::InterfaceName, source::Source};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct NodeId(pub String);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub data: Data,
    pub id: NodeId,
    pub position: Position,
    pub r#type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub function: FunctionName,
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    pub interface: Option<InterfaceName>,
    pub source: Source,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
