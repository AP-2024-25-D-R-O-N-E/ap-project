use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{network::NodeId, packet::Packet};

use crate::{
    fragmentation::Fragmenter,
    simulation_controller::structs::{ServerCommand, ServerEvent},
};

struct Server;

struct ServerOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<ServerEvent>,
    pub sim_contr_recv: Receiver<ServerCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl Fragmenter for Server {
    fn disassemble(
        msg: wg_2024::packet::Message,
    ) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> wg_2024::packet::Message {
        todo!()
    }
}

impl Server {
    pub fn new(options: ServerOptions) -> Self {
        todo!();
    }

    pub fn run() {
        todo!();
    }
}
