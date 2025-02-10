pub mod client_leonardo;
pub mod client_luca;
pub mod utils;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

use crossbeam::channel::{Receiver, Sender};
use tempfile::TempDir;
use wg_2024::{
    network::NodeId,
    packet::Packet,
};

use crate::simulation_controller::structs::{ClientCommand, ClientEvent};

// since we're doing the communication stuff, both clients will need to implement ClientTrait
pub trait ClientTrait {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ClientEvent>,
        sim_contr_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        temp_dir: Arc<TempDir>,
    ) -> Self
    where
        Self: Sized;

    fn run(&mut self);
}

impl Debug for dyn ClientTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client")
    }
}
