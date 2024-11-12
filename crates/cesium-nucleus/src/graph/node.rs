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

impl GraphNode {
    pub async fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        // add the size of the id
        bytes.push(self.id.len() as u8);
        bytes.extend(self.id.as_bytes());
        // add the size of the instructions
        let instr_bytes = self
            .instructions
            .iter()
            .fold(0, |acc, i| acc + i.to_bytes().len());
        bytes.extend(instr_bytes.to_le_bytes().iter());
        bytes.extend(self.instructions.iter().flat_map(|i| i.to_bytes()));

        let prev_nodes_bytes = self
            .prev_nodes
            .iter()
            .fold(0, |acc, n| acc + n.as_bytes().len());
        bytes.extend(prev_nodes_bytes.to_le_bytes().iter());
        bytes.extend(self.prev_nodes.iter().flat_map(|n| n.as_bytes()));
        bytes.extend(self.references.read().await.to_le_bytes().iter());
        bytes
    }

    pub async fn from_bytes(
        bytes: &[u8],
    ) -> Result<GraphNode, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = 0;

        // Read id length and id
        let id_len = bytes[cursor] as usize;
        cursor += 1;
        let id = String::from_utf8(bytes[cursor..cursor + id_len].to_vec())?;
        cursor += id_len;

        // Read instructions length and instructions
        let instr_bytes_len = usize::from_le_bytes(
            bytes[cursor..cursor + std::mem::size_of::<usize>()]
                .try_into()
                .map_err(|_| "Failed to read instructions length")?,
        );
        cursor += std::mem::size_of::<usize>();

        let mut instructions = Vec::new();
        let instr_end = cursor + instr_bytes_len;
        while cursor < instr_end {
            let (instruction, bytes_read) = Instruction::from_bytes(&bytes[cursor..instr_end])?;
            instructions.push(instruction);
            cursor += bytes_read;
        }

        // Read prev_nodes length and prev_nodes
        let prev_nodes_bytes_len = usize::from_le_bytes(
            bytes[cursor..cursor + std::mem::size_of::<usize>()]
                .try_into()
                .map_err(|_| "Failed to read prev_nodes length")?,
        );
        cursor += std::mem::size_of::<usize>();

        let mut prev_nodes = Vec::new();
        let mut bytes_processed = 0;
        while bytes_processed < prev_nodes_bytes_len {
            let node_str = String::from_utf8(
                bytes[cursor + bytes_processed..cursor + prev_nodes_bytes_len].to_vec(),
            )?;
            bytes_processed += node_str.as_bytes().len();
            prev_nodes.push(node_str);
        }
        cursor += prev_nodes_bytes_len;

        // Read references
        let references = u32::from_le_bytes(
            bytes[cursor..cursor + std::mem::size_of::<u32>()]
                .try_into()
                .map_err(|_| "Failed to read references")?,
        );

        Ok(GraphNode {
            id,
            instructions,
            prev_nodes,
            references: RwLock::new(references),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cesium_nebula::instruction::{Instruction, InstructionType};

    #[tokio::test]
    async fn test_graph_node() {
        let node = GraphNode {
            id: "node1".to_string(),
            instructions: vec![Instruction::new(
                InstructionType::CurrencyTransfer,
                vec![1, 2, 3],
            )],
            prev_nodes: vec!["node2".to_string()],
            references: RwLock::new(1),
        };

        let bytes = node.to_bytes().await;
        let node2 = GraphNode::from_bytes(&bytes).await.unwrap();

        assert_eq!(node.id, node2.id);
        assert_eq!(node.instructions, node2.instructions);
        assert_eq!(node.prev_nodes, node2.prev_nodes);
        assert_eq!(
            *node.references.read().await,
            *node2.references.read().await
        );
    }
}
