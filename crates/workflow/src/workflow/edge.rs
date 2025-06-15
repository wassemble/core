use serde::{Deserialize, Serialize};

use super::{InputName, NodeId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edge {
    pub input: InputName,
    pub source: NodeId,
    pub target: NodeId,
}
