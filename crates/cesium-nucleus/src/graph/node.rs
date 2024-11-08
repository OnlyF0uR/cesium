use cesium_nebula::instruction::Instruction;
use tokio::sync::RwLock;

pub type NodeId = String;

#[derive(Debug)]
pub struct GraphNode {
    pub id: NodeId,
    pub instructions: Vec<Instruction>,
    pub prev_nodes: Vec<NodeId>,
    pub references: RwLock<u32>,
}
