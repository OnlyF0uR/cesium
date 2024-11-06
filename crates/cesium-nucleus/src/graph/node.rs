use cesium_nebula::instruction::Instruction;
use tokio::sync::RwLock;

pub type NodeId = [u8; 48];

#[derive(Debug)]
pub struct GraphNode {
    pub id: NodeId,
    pub instructions: Vec<Instruction>,
    pub prev_nodes: Vec<NodeId>,
    pub confirmations: RwLock<u32>,
}
