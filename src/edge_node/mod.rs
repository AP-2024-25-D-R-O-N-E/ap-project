use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{controller::DroneCommand, network::NodeId, packet::Packet};

use crate::fragmentation::Fragmenter;

pub struct EdgeNodeOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<DroneCommand>,
    pub sim_contr_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub trait EdgeNode: Fragmenter {
    fn new(options: EdgeNodeOptions) -> Self;

    fn run();
}
