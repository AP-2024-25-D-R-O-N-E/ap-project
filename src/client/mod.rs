pub mod client_luca;
pub mod client_test_2;
pub mod client_test;

use std::{collections::HashMap, fmt::Debug};

use client_luca::ClientLuca;
use colored::Colorize;
use crossbeam::channel::{select_biased, Receiver, Sender};
use wg_2024::{
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

// since we're doing the communication stuff, both clients will need to implement ClientTrait
pub trait ClientTrait {
    fn new(
        id: NodeId,
        scs: Sender<ClientEvent>,
        scr: Receiver<ClientCommand>,
        packet_r: Receiver<Packet>,
        packet_s: HashMap<NodeId, Sender<Packet>>,
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
