use serde::{Deserialize, Serialize};

use super::node::NodeId;
use crate::metadata::InputName;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EdgeId(pub String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edge {
    pub id: EdgeId,
    pub input: InputName,
    pub source: NodeId,
    pub target: NodeId,
}
