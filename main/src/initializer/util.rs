use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    packet::Packet,
};

pub struct DroneChannels {
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<u8, Sender<Packet>>,
}

impl DroneChannels {
    pub fn new(
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<u8, Sender<Packet>>,
    ) -> DroneChannels {
        DroneChannels {
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
        }
    }
}
