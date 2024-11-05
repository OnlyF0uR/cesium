use tokio::sync::RwLock;

#[derive(Debug)]
pub struct NodeInput {
    pub instructions: Vec<Vec<u8>>,
    pub digest: Vec<u8>,
    pub reserved_gas: u128,
    pub priority_fee: u128,
}

pub type NodeId = [u8; 48];

#[derive(Debug)]
pub struct GraphNode {
    pub id: NodeId,
    pub instructions: Vec<Vec<u8>>,
    pub prev_nodes: Vec<NodeId>,
    pub confirmations: RwLock<u32>,
}
