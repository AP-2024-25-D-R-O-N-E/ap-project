use colored::Colorize;
use petgraph::{
    prelude::{StableGraph, StableUnGraph},
    Undirected,
};
use std::{io, thread::sleep, time::Duration};
use wg_2024::{
    network::SourceRoutingHeader,
    packet::{NackType, NodeType, PacketType},
};

use std::{collections::HashMap, thread::JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, Fragment, Nack, Packet},
};

use crate::{
    initializer::network_initializer::NetworkInitializer,
    simulation_controller::{edge::UiEdgePayload, node::UiNodePayload},
};

use super::structs::{ClientCommand, ClientEvent, ServerCommand, ServerEvent};

pub struct SimulationController {
    pub packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    pub node_event_channels: HashMap<NodeId, Receiver<DroneEvent>>,
    pub drone_command_channels: HashMap<NodeId, Sender<DroneCommand>>,
    pub client_event_channels: HashMap<NodeId, Receiver<ClientEvent>>,
    pub client_command_channels: HashMap<NodeId, Sender<ClientCommand>>,
    pub server_event_channels: HashMap<NodeId, Receiver<ServerEvent>>,
    pub server_command_channels: HashMap<NodeId, Sender<ServerCommand>>,

    pub default_msg_fragment: Packet,
    pub default_ack: Packet,
    pub default_nack: Packet,
    pub default_flood: Packet,

    pub topology: StableGraph<UiNodePayload, UiEdgePayload, Undirected>,
}

impl SimulationController {
    pub fn new(network_initializer: NetworkInitializer) -> SimulationController {
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
            default_msg_fragment: Packet {
                pack_type: PacketType::MsgFragment(Fragment {
                    fragment_index: 0,
                    total_n_fragments: 1,
                    length: 1,
                    data: [1; 128],
                }),
                routing_header: SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![0, 1],
                },
                session_id: 0,
            },
            default_ack: Packet {
                pack_type: PacketType::Ack(Ack { fragment_index: 0 }),
                routing_header: SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![0, 1],
                },
                session_id: 0,
            },
            default_nack: Packet {
                pack_type: PacketType::Nack(Nack {
                    fragment_index: 0,
                    nack_type: NackType::Dropped,
                }),
                routing_header: SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![0, 1],
                },
                session_id: 0,
            },
            default_flood: Packet {
                pack_type: PacketType::FloodRequest(FloodRequest {
                    flood_id: 0,
                    initiator_id: 0,
                    path_trace: vec![(0, NodeType::Client), (1, NodeType::Drone)],
                }),
                routing_header: SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![0, 1],
                },
                session_id: 0,
            },
            topology: network_initializer.topology,
        }
    }

    fn prompt_id_and_execute(prompt: String, action: impl Fn(NodeId)) {
        println!("{}", prompt.italic());
        loop {
            let mut buffer = String::new();
            let input = io::stdin().read_line(&mut buffer);
            match (input, buffer.trim().parse::<NodeId>()) {
                (Ok(_), Ok(node_id)) => {
                    println!();
                    action(node_id);
                    sleep(Duration::from_millis(100));
                    println!();
                    break;
                }
                _ => println!("{}", "Insert a valid number".italic().yellow()),
            }
        }
    }

    // launches a text user interface to launch functions in real time
    pub fn run_tui(&self) {
        loop {
            println!(" {}) {}", "1".italic(), "Send fragment".blue());
            println!(" {}) {}", "2".italic(), "Send ack".blue());
            println!(" {}) {}", "3".italic(), "Send nack".blue());
            println!(" {}) {}", "4".italic(), "Send flood request".blue());

            println!("{}) {}", "-1".italic(), "Quit".blue());
            println!("{}", "Please enter a number: ".italic());

            let mut buffer = String::new();
            let input = io::stdin().read_line(&mut buffer);
            match &input {
                Ok(input) => {
                    match buffer.trim().parse::<i32>() {
                        Ok(action_number) => match action_number {
                            1 => SimulationController::prompt_id_and_execute(
                                "Insert a node id".to_string(),
                                |id| self.send_default_msg_fragment(id),
                            ),
                            2 => SimulationController::prompt_id_and_execute(
                                "Insert a node id".to_string(),
                                |id| self.send_default_ack(id),
                            ),
                            3 => SimulationController::prompt_id_and_execute(
                                "Insert a node id".to_string(),
                                |id| self.send_default_nack(id),
                            ),
                            4 => SimulationController::prompt_id_and_execute(
                                "Insert a node id".to_string(),
                                |id| self.send_default_flood_request(id),
                            ),
                            -1 => break,
                            _ => println!("{}", "Insert a valid number".italic().yellow()),
                        },
                        Err(err) => log::error!("Please insert a valid number"),
                    };
                }
                Err(err) => log::error!("Cannot read from stdin: {}", err),
            }
        }
    }
}
