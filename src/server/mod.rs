pub mod server_test;

use std::{collections::HashMap, fmt::{write, Debug}};

use colored::Colorize;
use crossbeam::channel::{select_biased, Receiver, Sender};
use wg_2024::{
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ServerCommand, ServerEvent},
};

pub trait ServerTrait {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ServerEvent>,
        sim_contr_recv: Receiver<ServerCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self where Self:Sized;

    fn run(&mut self);
}

/// this would need an actual definition done in some way or another
impl Debug for dyn ServerTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server")
    }
}