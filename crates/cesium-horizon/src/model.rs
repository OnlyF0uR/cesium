use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use cesium_crypto::keys::PublicKeyBytes;
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::mpsc,
};

#[derive(Clone, Debug)]
pub struct Packet {
    pub id: u64,
    pub data: Vec<u8>,
    pub retransmit_count: u32,
    pub origin: String,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: PublicKeyBytes,
    pub layer: u32,
    pub neighbors: Arc<Mutex<Vec<String>>>,
    pub received_packets: Arc<Mutex<HashSet<u64>>>,
    pub tx: mpsc::Sender<Packet>,
    pub udp_socket: Arc<UdpSocket>,
    pub tcp_connection: Arc<Mutex<HashMap<String, TcpStream>>>,
}
