use colored::Colorize;
use std::io;

use std::{collections::HashMap, thread::JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, NodeEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, Fragment, Nack, Packet},
};

use crate::initializer::network_initializer::NetworkInitializer;

use super::structs::{ClientCommand, ClientEvent, ServerCommand, ServerEvent};

pub struct SimulationController {
    pub packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    pub node_event_channels: HashMap<NodeId, Receiver<NodeEvent>>,
    pub drone_command_channels: HashMap<NodeId, Sender<DroneCommand>>,
    pub client_event_channels: HashMap<NodeId, Receiver<ClientEvent>>,
    pub client_command_channels: HashMap<NodeId, Sender<ClientCommand>>,
    pub server_event_channels: HashMap<NodeId, Receiver<ServerEvent>>,
    pub server_command_channels: HashMap<NodeId, Sender<ServerCommand>>,
    pub default_msg_fragment: Packet,
    pub default_ack: Packet,
    pub default_nack: Packet,
    pub default_flood: Packet,
}

impl SimulationController {
    pub fn new(network_initializer: &NetworkInitializer) -> SimulationController {
        SimulationController {
            packet_channels: network_initializer.packet_channels.clone(),
            node_event_channels: network_initializer
                .node_event_channels
                .clone()
                .iter()
                .map(|(node_id, channel)| (*node_id, channel.1.clone()))
                .collect(),
            drone_command_channels: network_initializer
                .drone_command_channels
                .clone()
                .iter()
                .map(|(node_id, channel)| (*node_id, channel.0.clone()))
                .collect(),

            client_event_channels: network_initializer
                .client_event_channels
                .clone()
                .iter()
                .map(|(client_id, channel)| (*client_id, channel.1.clone()))
                .collect(),
            client_command_channels: network_initializer
                .client_command_channels
                .clone()
                .iter()
                .map(|(node_id, channel)| (*node_id, channel.0.clone()))
                .collect(),

            server_event_channels: network_initializer
                .server_event_channels
                .clone()
                .iter()
                .map(|(server_id, channel)| (*server_id, channel.1.clone()))
                .collect(),
            server_command_channels: network_initializer
                .server_command_channels
                .clone()
                .iter()
                .map(|(node_id, channel)| (*node_id, channel.0.clone()))
                .collect(),
            default_msg_fragment: todo!(),
            default_ack: todo!(),
            default_nack: todo!(),
            default_flood: todo!(),
        }
    }

    pub fn send_default_msg_fragment(&self, node_id: NodeId) {
        self.send_msg_fragment(self.default_msg_fragment.clone(), node_id);
    }

    pub fn send_msg_fragment(&self, packet: Packet, node_id: NodeId) {
        match &self.packet_channels.get(&node_id) {
            Some(channel) => {
                channel.0.send(packet);
                log::info!("Sending to node {}", node_id)
            }
            None => {
                log::error!("Specified node does not exist")
            }
        }
    }

    // launches a text user interface to launch functions in real time
    pub fn run_tui() {
        let mut buffer = String::new();
        let input = io::stdin().read_line(&mut buffer);

        println!("{}", "Please enter a number: ".italic());
        loop {
            match &input {
                Ok(input) => {
                    match buffer.parse::<i32>() {
                        Ok(action_number) => match action_number {
                            1 => println!("prova"),
                            -1 => break,
                            _ => (),
                        },
                        Err(err) => log::error!("Please insert a valid number"),
                    };
                }
                Err(err) => log::error!("Cannot read from stdin: {}", err),
            }
        }
    }
}

// impl SimulationController {
//     fn new(network_initializer: &NetworkInitializer) -> SimulationController {
//     }
// }
