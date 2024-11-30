use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{network::NodeId, packet::Packet};

use crate::{fragmentation::Fragmenter, simulation_controller::structs::{EdgeNodeCommand, EdgeNodeEvent}};

pub struct EdgeNodeOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<EdgeNodeEvent>,
    pub sim_contr_recv: Receiver<EdgeNodeCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub trait EdgeNode: Fragmenter {
    fn new(options: EdgeNodeOptions) -> Self;

    fn run();
}
