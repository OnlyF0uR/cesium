use std::{collections::HashSet, sync::Arc};

use dashmap::DashMap;
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::{mpsc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Packet {
    pub id: u64,
    pub data: Vec<u8>,
    pub retransmit_count: u32,
    pub origin: String,
}

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub layer: u32,
    pub neighbors_ips: Arc<Mutex<Vec<String>>>,
    pub received_packets: Arc<Mutex<HashSet<u64>>>,
    pub tx: mpsc::Sender<Packet>,
    pub udp_socket: Arc<UdpSocket>,
    pub tcp_connections: DashMap<String, TcpStream>,
}
