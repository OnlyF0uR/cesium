use cesium_crypto::keys::PublicKeyBytes;
use tokio::sync::mpsc;

pub struct HorizonNetwork {
    nodes: HashMap<PublicKeyBytes, Node>,
    layers: Vec<Vec<PublicKeyBytes>>,
    fanout: usize,
    multicast_addr: String,
}

impl HorizonNetwork {
    pub fn new(fanout: usize, mc_addr: &str) -> Self {
        HorizonNetwork {
            nodes: HashMap::new(),
            layers: Vec::new(),
            fanout,
            multicast_addr: mc_addr.to_string(),
        }
    }

    async fn add_node(&mut self, id: String, layer: u32) -> mpsc::Receiver<Packet> {
        let (tx, rx) = mpsc::channel(100);
        let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", rand::random::<u16>()))
            .await
            .unwrap();
        udp_socket
            .join_multicast_v4(
                &self.multicast_addr.parse().unwrap(),
                &"0.0.0.0".parse().unwrap(),
            )
            .await
            .unwrap();

        let node = Node {
            id: id.clone(),
            layer,
            neighbors: Arc::new(Mutex::new(Vec::new())),
            received_packets: Arc::new(Mutex::new(HashSet::new())),
            tx,
            udp_socket: Arc::new(udp_socket),
            tcp_connections: Arc::new(Mutex::new(HashMap::new())),
        };

        // Extend layers if necessary
        while self.layers.len() <= layer as usize {
            self.layers.push(Vec::new());
        }
        self.layers[layer as usize].push(id.clone());
        self.nodes.insert(id, node);

        // Recalculate neighborhoods
        self.rebuild_neighborhoods();
        rx
    }

    fn rebuild_neighborhoods(&mut self) {
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
                    *node.neighbors.lock().unwrap() = neighbors;
                }
            }
        }
    }

    async fn broadcast(
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
                    // UDP multicast attempt
                    if let Err(e) = node
                        .udp_socket
                        .send_to(&packet.data, &self.multicast_addr)
                        .await
                    {
                        eprintln!("UDP multicast error: {}", e);
                    }

                    // TCP fallback
                    let mut tcp_connections = node.tcp_connections.lock().await;
                    for neighbor in node.neighbors.lock().await.iter() {
                        let conn = tcp_connections
                            .entry(neighbor.clone())
                            .or_try_insert_with(|| async {
                                TcpStream::connect(format!("{}:{}", neighbor, 12345)).await
                            })
                            .await?;

                        if let Err(e) = conn.write_all(&packet.data).await {
                            eprintln!("TCP send error: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

async fn handle_packets(mut node: Node, mut rx: mpsc::Receiver<Packet>) {
    while let Some(packet) = rx.recv().await {
        let packet_id = packet.id;

        // Check if we've seen this packet before
        let should_process = {
            let mut received = node.received_packets.lock().unwrap();
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
            let neighbors = node.neighbors.lock().unwrap().clone();

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
                    let mut tcp_connections = node.tcp_connections.lock().unwrap();
                    if !tcp_connections.contains_key(&node.id) {
                        let tcp_stream = TcpStream::connect(format!("{}:{}", node.id, 12345))
                            .await
                            .unwrap();
                        tcp_connections.insert(node.id.clone(), tcp_stream);
                    }
                    tcp_connections
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
