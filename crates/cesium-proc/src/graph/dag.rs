use cesium_material::{
    id::generate_id,
    keys::{slice_to_array_48, SIG_BYTE_LEN},
};
use dashmap::DashMap;
use std::sync::Arc;

use super::{
    errors::GraphError,
    node::{GraphNode, NodeInput},
};

pub struct Graph {
    nodes: Arc<DashMap<[u8; 48], Arc<GraphNode>>>,
    pack_iv_count: usize,
    pack_min_conf: u32,
    pack_proportion: f32,
}

impl Graph {
    pub fn new(pack_iv_count: usize, pack_min_conf: u32, pack_proportion: f32) -> Self {
        Self {
            nodes: Arc::new(DashMap::new()),
            pack_iv_count,
            pack_min_conf,
            pack_proportion,
        }
    }

    pub fn default() -> Self {
        Self::new(2500, 5, 0.45)
    }

    // The minimal amount of nodes required to kick off the graph is
    pub async fn add_genesis(&self, input: &NodeInput) -> Result<(), GraphError> {
        if let Err(e) = self.validate_item(input) {
            return Err(e);
        }

        // Compute random node_id of 48 characters
        let node_id = generate_id();
        let node_id = match slice_to_array_48(node_id.as_slice()) {
            Ok(id) => *id,
            // This should never happen
            Err(_) => return Err(GraphError::InvalidNodeId),
        };

        let node = GraphNode {
            id: node_id,
            instructions: input.instructions.clone(),
            prev_nodes: vec![],
            confirmations: 0.into(),
        };
        let node_arc: Arc<GraphNode> = Arc::new(node);

        // Add node to the graph
        self.nodes.insert(node_id, node_arc);
        Ok(())
    }

    pub async fn add_item(&self, input: &NodeInput) -> Result<(), GraphError> {
        if let Err(e) = self.validate_item(input) {
            return Err(e);
        }

        // Compute random node_id of 48 characters
        let node_id = generate_id();
        let node_id = match slice_to_array_48(node_id.as_slice()) {
            Ok(id) => *id,
            // This should never happen
            Err(_) => return Err(GraphError::InvalidNodeId),
        };

        let ref_nodes = self.get_pending_nodes().await;
        if ref_nodes.is_empty() {
            return Err(GraphError::MissingGenesisNode);
        }

        for ref_node in &ref_nodes {
            self.validate_node(ref_node).await?;
        }

        let node = GraphNode {
            id: node_id,
            instructions: input.instructions.clone(),
            prev_nodes: ref_nodes.iter().map(|n| n.id).collect(),
            confirmations: 0.into(),
        };
        let node_arc = Arc::new(node);

        // Add node to the graph
        self.nodes.insert(node_id, node_arc);

        // if nodes length
        if self.nodes.len() >= self.pack_iv_count {
            self.pack_history().await?;
        }

        Ok(())
    }

    pub fn set_interval_count(&mut self, count: usize) {
        self.pack_iv_count = count;
    }

    pub fn set_min_confirmations(&mut self, count: u32) {
        self.pack_min_conf = count;
    }

    pub fn set_proportion(&mut self, proportion: f32) {
        self.pack_proportion = proportion;
    }

    async fn pack_history(&self) -> Result<(), GraphError> {
        // Get all nodes with 5 or more confirmations
        let nodes = self.get_packable_nodes().await;
        println!("Nodes to pack: {:?}", nodes);

        // TODO: This
        // This function will get the 45% of nodes with the most confirmations provided
        // they have 5 or more confirmations, then writes them to rocksdb and removes them
        // from memory
        Ok(())
    }

    async fn get_packable_nodes(&self) -> Vec<Arc<GraphNode>> {
        let packable_count = (self.nodes.len() as f32 * self.pack_proportion).ceil() as usize;
        self.get_nodes_with_sorting(true, packable_count).await
    }

    async fn get_pending_nodes(&self) -> Vec<Arc<GraphNode>> {
        self.get_nodes_with_sorting(false, 5).await
    }

    fn validate_item(&self, input: &NodeInput) -> Result<(), GraphError> {
        // Some very basic validation
        if input.instructions.is_empty() {
            return Err(GraphError::InvalidNodeInput);
        }

        if input.digest.len() < SIG_BYTE_LEN {
            return Err(GraphError::InvalidNodeInput);
        }

        // TODO: Signature check

        // TODO: Instruction validity check (balances, enough reserved gas, etc.)

        Ok(())
    }

    async fn validate_node(&self, node: &GraphNode) -> Result<(), GraphError> {
        // TODO: Validate the current node

        // Get a read lock on the node's previous nodes
        let prev_nodes: Vec<Arc<GraphNode>> = {
            node.prev_nodes
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .map(|node| node.value().clone())
                .collect()
        };

        // Acquire write locks on the confirmations of the previous nodes
        let mut prev_nodes_confirmation_locks = Vec::with_capacity(prev_nodes.len());
        for prev_node in &prev_nodes {
            prev_nodes_confirmation_locks.push(prev_node.confirmations.write().await);
        }

        // Update the confirmations of the previous nodes
        for mut confirmations_lock in prev_nodes_confirmation_locks {
            *confirmations_lock += 1;
        }

        Ok(())
    }

    async fn get_nodes_with_sorting(&self, take_top: bool, limit: usize) -> Vec<Arc<GraphNode>> {
        // Preallocate vector with known capacity
        let mut nodes_with_confirmations = Vec::with_capacity(self.nodes.len());

        // Collect nodes and confirmations
        for node in self.nodes.iter() {
            let confirmations = *node.confirmations.read().await;
            nodes_with_confirmations.push((node.clone(), confirmations));
        }

        // Sort by confirmation count
        nodes_with_confirmations.sort_by(|a, b| a.1.cmp(&b.1));

        // Take from either end of the sorted list depending on take_top
        let iter: Box<dyn Iterator<Item = &(Arc<GraphNode>, u32)>> = if take_top {
            Box::new(nodes_with_confirmations.iter().rev().take(limit))
        } else {
            Box::new(nodes_with_confirmations.iter().take(limit))
        };

        iter.map(|(node, _)| node.clone()).collect()
    }
}
