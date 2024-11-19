use cesium_crypto::keys::{Account, SIG_BYTE_LEN};
use cesium_nebula::transaction::Transaction;
use dashmap::DashMap;
use std::sync::Arc;

use super::{
    errors::GraphError,
    node::{GraphNode, NodeId},
};

pub struct Graph<'a> {
    account: &'a Account,
    nodes: Arc<DashMap<NodeId, Arc<GraphNode>>>,
    pack_iv_count: usize,
    pack_min_conf: u32,
    pack_proportion: f32,
}

impl<'a> Graph<'a> {
    pub fn new(
        account: &'a Account,
        pack_iv_count: usize,
        pack_min_conf: u32,
        pack_proportion: f32,
    ) -> Self {
        Self {
            account,
            nodes: Arc::new(DashMap::new()),
            pack_iv_count,
            pack_min_conf,
            pack_proportion,
        }
    }

    pub fn default(account: &'a Account) -> Self {
        Self::new(account, 2500, 5, 0.45)
    }

    // The minimal amount of nodes required to kick off the graph is
    pub async fn add_genesis(&self, input: &Transaction) -> Result<(), GraphError> {
        if let Err(e) = self.validate_item(input) {
            return Err(e);
        }

        // Compute random node_id of 48 characters
        let node_id = input.create_id()?;
        let node = GraphNode {
            id: node_id.clone(),
            instructions: input.instructions.clone(),
            prev_nodes: vec![],
            references: 0.into(),
        };
        let node_arc: Arc<GraphNode> = Arc::new(node);

        // Add node to the graph
        self.nodes.insert(node_id, node_arc);
        Ok(())
    }

    pub async fn add_item(&self, input: &Transaction) -> Result<(), GraphError> {
        if let Err(e) = self.validate_item(input) {
            return Err(e);
        }

        // Compute random node_id of 48 characters
        let node_id = input.create_id()?;

        let ref_nodes = self.get_pending_nodes().await;
        if ref_nodes.is_empty() {
            return Err(GraphError::MissingGenesisNode);
        }

        for ref_node in &ref_nodes {
            self.validate_node(ref_node).await?;
        }

        let node = GraphNode {
            id: node_id.clone(),
            instructions: input.instructions.clone(),
            prev_nodes: ref_nodes.iter().map(|n| n.id.clone()).collect(),
            references: 0.into(),
        };
        let node_arc = Arc::new(node);

        // Add node to the graph
        self.nodes.insert(node_id, node_arc);
        // TODO: Gossip the node to other validators

        // if nodes length
        if self.nodes.len() >= self.pack_iv_count {
            self.pack_history().await?;
        }

        Ok(())
    }

    pub fn set_interval_count(&mut self, count: usize) {
        self.pack_iv_count = count;
    }

    pub fn set_min_references(&mut self, count: u32) {
        self.pack_min_conf = count;
    }

    pub fn set_proportion(&mut self, proportion: f32) {
        self.pack_proportion = proportion;
    }

    async fn pack_history(&self) -> Result<(), GraphError> {
        // Get all nodes with 5 or more confirmed references
        let nodes = self.get_packable_nodes().await;
        // println!("Nodes to pack: {:?}", nodes);

        // Convert the nodes to bytes
        let msg = futures::future::join_all(
            nodes
                .iter()
                .map(|node: &Arc<GraphNode>| async { node.to_bytes().await }),
        )
        .await
        .concat();

        // Sign the message
        let sig = self
            .account
            .digest(&msg)
            .map_err(|e| GraphError::SigningError(e))?;

        // TODO: This
        // Broadcast the checkpoint to other validators

        if let Err(e) = cesium_storage::RocksDBStore::instance().put(&sig, &msg) {
            return Err(GraphError::PutCheckpointError(e));
        }

        // Remove nodes from memory
        for node in nodes {
            self.nodes.remove(&node.id);
        }

        Ok(())
    }

    async fn get_packable_nodes(&self) -> Vec<Arc<GraphNode>> {
        let packable_count = (self.nodes.len() as f32 * self.pack_proportion).ceil() as usize;
        self.get_nodes_with_sorting(true, packable_count).await
    }

    async fn get_pending_nodes(&self) -> Vec<Arc<GraphNode>> {
        self.get_nodes_with_sorting(false, 5).await
    }

    fn validate_item(&self, input: &Transaction) -> Result<(), GraphError> {
        if input.digest.is_none() {
            return Err(GraphError::MissingSignature);
        }

        // Some very basic validation
        if input.instructions.is_empty() {
            return Err(GraphError::InvalidNodeInput);
        }

        let sig = input.digest.as_ref().unwrap();
        if sig.len() < SIG_BYTE_LEN {
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

        // Acquire write locks on the references of the previous nodes
        let mut prev_nodes_confirmation_locks = Vec::with_capacity(prev_nodes.len());
        for prev_node in &prev_nodes {
            prev_nodes_confirmation_locks.push(prev_node.references.write().await);
        }

        // Update the references of the previous nodes
        for mut references_lock in prev_nodes_confirmation_locks {
            *references_lock += 1;
        }

        Ok(())
    }

    async fn get_nodes_with_sorting(&self, take_top: bool, limit: usize) -> Vec<Arc<GraphNode>> {
        // Preallocate vector with known capacity
        let mut nodes_with_references = Vec::with_capacity(self.nodes.len());

        // Collect nodes and references
        for node in self.nodes.iter() {
            let references = *node.references.read().await;
            nodes_with_references.push((node.clone(), references));
        }

        // Sort by confirmation count
        nodes_with_references.sort_by(|a, b| a.1.cmp(&b.1));

        // Take from either end of the sorted list depending on take_top
        let iter: Box<dyn Iterator<Item = &(Arc<GraphNode>, u32)>> = if take_top {
            Box::new(nodes_with_references.iter().rev().take(limit))
        } else {
            Box::new(nodes_with_references.iter().take(limit))
        };

        iter.map(|(node, _)| node.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cesium_crypto::keys::Account;
    use cesium_nebula::instruction::{Instruction, InstructionType};
    use std::sync::Arc;
    use tokio::task;

    #[tokio::test]
    async fn test_add_valid_transaction() {
        let acc: Account = Account::create();
        let dag = Graph::default(&acc);

        let tx = create_valid_transaction(&acc);
        dag.add_genesis(&tx).await.unwrap();

        assert_eq!(dag.nodes.len(), 1);
        assert_valid_node(&dag, tx.instructions.clone());
    }

    #[tokio::test]
    async fn test_add_transaction_with_missing_signature() {
        let acc = Account::create();
        let dag = Graph::default(&acc);

        let mut tx = Transaction::new(18000, 0);
        tx.add_instruction(Instruction::new(
            InstructionType::CurrencyTransfer,
            Vec::new(),
        ))
        .unwrap();

        let result = dag.add_genesis(&tx).await;
        assert!(result.is_err());
        assert_eq!(dag.nodes.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_transaction_addition() {
        let acc = Arc::new(Account::create());
        let dag: Arc<Graph<'_>> = Arc::new(Graph::default(Box::leak(Box::new(acc.clone()))));

        dag.add_genesis(&create_valid_transaction(&acc))
            .await
            .unwrap();

        let mut handles = Vec::with_capacity(10);
        for _ in 0..10 {
            let dag = Arc::clone(&dag);
            let acc = Arc::clone(&acc);
            handles.push(task::spawn(async move {
                let tx = create_valid_transaction(&acc);
                dag.add_item(&tx).await.unwrap();
            }));
        }

        tokio::join!(async {
            for handle in handles {
                handle.await.unwrap();
            }
        });

        assert_eq!(dag.nodes.len(), 11);
    }

    #[tokio::test]
    async fn test_pack_history() {
        let acc = Account::create();
        let mut dag = Graph::default(&acc);
        dag.set_interval_count(5);
        dag.set_min_references(3);
        dag.set_proportion(0.4);

        // Add genesis node
        dag.add_genesis(&create_valid_transaction(&acc))
            .await
            .unwrap();

        for _ in 0..10 {
            let tx = create_valid_transaction(&acc);
            dag.add_item(&tx).await.unwrap();
        }

        // Confirm some nodes to trigger packing
        for node in dag.nodes.iter() {
            let mut lock = node.value().references.write().await;
            *lock = 3;
        }

        dag.pack_history().await.unwrap();
        assert_eq!(dag.nodes.len(), 1);
    }

    fn create_valid_transaction(acc: &Account) -> Transaction {
        let mut tx = Transaction::new(18000, 0);
        tx.add_instruction(Instruction::new(
            InstructionType::CurrencyTransfer,
            Vec::new(),
        ))
        .unwrap();
        tx.sign(acc).unwrap();
        tx
    }

    fn assert_valid_node(dag: &Graph, instructions: Vec<Instruction>) {
        let node = dag.nodes.iter().next().unwrap().value().clone();
        assert_eq!(node.instructions.len(), instructions.len());
        // Add more assertions about the node structure and content
    }
}
