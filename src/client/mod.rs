use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{network::NodeId, packet::Packet};

use crate::{
    fragmentation::Fragmenter,
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

struct Client;

struct ClientOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<ClientEvent>,
    pub sim_contr_recv: Receiver<ClientCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl Fragmenter for Client {
    fn disassemble(
        msg: wg_2024::packet::Message,
    ) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> wg_2024::packet::Message {
        todo!()
    }
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        todo!();
    }

    pub fn run() {
        todo!();
    }
}
