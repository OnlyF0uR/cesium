use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use dashmap::DashMap;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
    sync::{mpsc, Mutex},
    time::sleep,
};

use crate::model::{Node, Packet};

pub struct HorizonNetwork {
    nodes: HashMap<String, Node>,
    layers: Vec<Vec<String>>,
    fanout: usize,
    multicast_addr: String,
    multicast_port: u16,
}

impl HorizonNetwork {
    pub fn new(fanout: usize, mc_addr: &str, mc_port: u16) -> Self {
        HorizonNetwork {
            nodes: HashMap::new(),
            layers: Vec::new(),
            fanout,
            multicast_addr: mc_addr.to_string(),
            multicast_port: mc_port,
        }
    }

    pub async fn add_node(&mut self, id: String, layer: u32) -> mpsc::Receiver<Packet> {
        let (tx, rx) = mpsc::channel(100);
        let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", rand::random::<u16>()))
            .await
            .unwrap();

        let multicast_addr = Ipv4Addr::from_str(&self.multicast_addr).unwrap();
        let interface_addr = Ipv4Addr::from_str("0.0.0.0").unwrap();

        udp_socket
            .join_multicast_v4(multicast_addr, interface_addr)
            .unwrap();

        let node = Node {
            id: id.clone(),
            layer,
            neighbors_ips: Arc::new(Mutex::new(Vec::new())),
            received_packets: Arc::new(Mutex::new(HashSet::new())),
            tx,
            udp_socket: Arc::new(udp_socket),
            tcp_connections: DashMap::new(),
        };

        // Extend layers if necessary
        while self.layers.len() <= layer as usize {
            self.layers.push(Vec::new());
        }
        self.layers[layer as usize].push(id.clone());
        self.nodes.insert(id, node);

        // Recalculate neighborhoods
        self.rebuild_neighborhoods().await;
        rx
    }

    async fn rebuild_neighborhoods(&mut self) {
        for layer in 0..self.layers.len() {
            for node_idx in 0..self.layers[layer].len() {
                let node_id = self.layers[layer][node_idx].clone();
                let mut neighbors = Vec::new();

                // Add nodes from next layer based on fanout
                if layer + 1 < self.layers.len() {
                    let start_idx = node_idx * self.fanout;
                    let end_idx = (node_idx + 1) * self.fanout;

                    for target_idx in start_idx..end_idx {
                        if target_idx < self.layers[layer + 1].len() {
                            neighbors.push(self.layers[layer + 1][target_idx].clone());
                        }
                    }
                }

                if let Some(node) = self.nodes.get(&node_id) {
                    *node.neighbors_ips.lock().await = neighbors;
                }
            }
        }
    }

    pub async fn broadcast(
        &self,
        origin: String,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let packet = Packet {
            id: rand::random(),
            data,
            retransmit_count: 0,
            origin,
        };

        if let Some(first_layer) = self.layers.first() {
            for node_id in first_layer {
                if let Some(node) = self.nodes.get(node_id) {
                    let full_addr = format!("{}:{}", self.multicast_addr, self.multicast_port);
                    let socket_addr = SocketAddr::from_str(&full_addr).unwrap();

                    // UDP multicast attempt
                    if let Err(e) = node.udp_socket.send_to(&packet.data, socket_addr).await {
                        eprintln!("UDP multicast error: {}", e);
                    }

                    // TCP fallback
                    for neightbour_ip in node.neighbors_ips.lock().await.iter() {
                        let con = node.tcp_connections.get(neightbour_ip);
                        if con.is_none() {
                            let tcp_stream =
                                TcpStream::connect(format!("{}:{}", neightbour_ip, 12345)).await?;
                            node.tcp_connections
                                .insert(neightbour_ip.clone(), tcp_stream);
                        }

                        let mut con = node.tcp_connections.get_mut(neightbour_ip).unwrap();
                        if let Err(e) = con.write_all(&packet.data).await {
                            eprintln!("TCP send error: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_packets(&self, node: Node, mut rx: mpsc::Receiver<Packet>) {
        while let Some(packet) = rx.recv().await {
            let packet_id = packet.id;

            // Check if we've seen this packet before
            let should_process = {
                let mut received = node.received_packets.lock().await;
                if !received.contains(&packet_id) {
                    received.insert(packet_id);
                    true
                } else {
                    false
                }
            };

            if should_process {
                // Process packet here
                // println!(
                //     "Node {} received packet {} from {}",
                //     node.id, packet.id, packet.origin
                // );

                // Get neighbors to forward to
                let neighbors = node.neighbors_ips.lock().await.clone();

                // Add small random delay to prevent network congestion
                sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;

                // Forward to neighbors via UDP multicast and TCP fallback
                let mut new_packet = packet.clone();
                new_packet.retransmit_count += 1;

                for neighbor in neighbors {
                    if let Some(node) = self.nodes.get(&neighbor) {
                        // Try UDP multicast first
                        node.udp_socket
                            .send_to(&new_packet.data, &self.multicast_addr)
                            .await
                            .unwrap();

                        // Fall back to TCP if necessary
                        if !node.tcp_connections.contains_key(&node.id) {
                            let tcp_stream = TcpStream::connect(format!("{}:{}", node.id, 12345))
                                .await
                                .unwrap();
                            node.tcp_connections.insert(node.id.clone(), tcp_stream);
                        }
                        node.tcp_connections
                            .get_mut(&node.id)
                            .unwrap()
                            .write_all(&new_packet.data)
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_add_node() {
        let mut network = HorizonNetwork::new(3, "224.0.0.5", 12345);
        network.add_node("node1".to_string(), 0).await;
        assert!(network.nodes.contains_key("node1"));
        assert_eq!(network.layers[0][0], "node1");
    }

    #[tokio::test]
    async fn test_rebuild_neighborhoods() {
        let mut network = HorizonNetwork::new(3, "224.0.0.5", 12345);
        network.add_node("node1".to_string(), 0).await;
        network.add_node("node2".to_string(), 0).await;
        network.add_node("node3".to_string(), 1).await;
        network.add_node("node4".to_string(), 1).await;
        network.add_node("node5".to_string(), 1).await;

        network.rebuild_neighborhoods().await;

        assert_eq!(network.layers[0].len(), 2);
        assert_eq!(network.layers[1].len(), 3);

        if let Some(node1) = network.nodes.get("node1") {
            assert_eq!(node1.neighbors_ips.lock().await.len(), 3);
        } else {
            panic!("node1 not found");
        }
    }

    #[tokio::test]
    async fn test_broadcast_invalid_host() {
        let mut network = HorizonNetwork::new(3, "224.0.0.5", 12345);
        network.add_node("node1".to_string(), 0).await;
        network.add_node("node2".to_string(), 0).await;
        network.add_node("node3".to_string(), 1).await;

        // This should fail due to unknown host within 3 seconds,
        // but just in case, we'll set a 3 second timeout
        let result = timeout(
            Duration::from_secs(3),
            network.broadcast("node1".to_string(), vec![1, 2, 3]),
        )
        .await;
        assert!(result.is_ok());

        let result = result.unwrap();

        // let result = network.broadcast("node1".to_string(), vec![1, 2, 3]).await;
        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.to_string(), "No such host is known. (os error 11001)");
    }
}
