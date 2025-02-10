use std::{
    collections::HashMap,
    sync::{Arc, Barrier},
};

use crossbeam::channel::unbounded;
use wg_2024::{
    controller::*,
    network::NodeId,
    packet::{Packet, PacketType},
};

use crate::initializer::{
    drone_vendor::DroneVendor,
    network_initializer::spawn_drone_thread_by_vendor,
};

use super::{simulation_controller::SimulationController, ClientCommand, ServerCommand};

impl SimulationController {
    pub fn send_default_msg_fragment(&self, node_id: NodeId) {
        self.send_msg_fragment(self.default_msg_fragment.clone());
    }

    pub fn send_msg_fragment(&self, packet: Packet) {
        match packet.routing_header.current_hop() {
            Some(current_hop) => match &self.packet_channels.get(&current_hop) {
                Some(channel) => match &packet.pack_type {
                    PacketType::MsgFragment(_) => {
                        channel.0.send(packet);
                        log::info!("Sending to node {}", current_hop)
                    }
                    _ => {
                        log::error!("Provided package is not a msg fragment")
                    }
                },
                None => {
                    log::error!("Specified node does not exist")
                }
            },
            None => log::error!("Cannot get current hop"),
        }
    }

    pub fn send_default_flood_request(&self, node_id: NodeId) {
        self.send_flood_request(self.default_flood.clone(), 1);
    }

    pub fn send_flood_request(&self, packet: Packet, node: NodeId) {
        match packet.pack_type.clone() {
            PacketType::FloodRequest(flood_req) => match &self.packet_channels.get(&node) {
                Some(channel) => {
                    channel.0.send(packet);
                    log::info!(
                        "Sending to node {}, initiated by {}",
                        node,
                        flood_req.initiator_id
                    )
                }
                None => {
                    log::error!("Specified node does not exist")
                }
            },
            _ => {
                log::error!("Provided package is not a flood request")
            }
        }
    }

    pub fn send_default_ack(&self, node_id: NodeId) {
        self.send_ack(self.default_ack.clone());
    }

    pub fn send_ack(&self, packet: Packet) {
        match packet.routing_header.current_hop() {
            Some(current_hop) => match &self.packet_channels.get(&current_hop) {
                Some(channel) => match &packet.pack_type {
                    PacketType::Ack(_) => {
                        channel.0.send(packet);
                        log::info!("Sending to node {}", current_hop)
                    }
                    _ => {
                        log::error!("Provided package is not an ack")
                    }
                },
                None => {
                    log::error!("Specified node does not exist")
                }
            },

            None => log::error!("Cannot get current hop"),
        }
    }

    pub fn send_default_nack(&self, node_id: NodeId) {
        self.send_nack(self.default_nack.clone());
    }

    pub fn send_nack(&self, packet: Packet) {
        match packet.routing_header.current_hop() {
            Some(current_hop) => match &self.packet_channels.get(&current_hop) {
                Some(channel) => match &packet.pack_type {
                    PacketType::Nack(_) => {
                        channel.0.send(packet);
                        log::info!("Sending to node {}", current_hop)
                    }
                    _ => {
                        log::error!("Provided package is not an nack")
                    }
                },
                None => {
                    log::error!("Specified node does not exist")
                }
            },

            None => log::error!("Cannot get current hop"),
        }
    }

    pub fn send_crash_command(&self, node_id: NodeId) {
        match &self.drone_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(DroneCommand::Crash);
            }
            None => {
                log::error!("Specified node does not exist");
            }
        }
    }

    pub fn send_set_pdr_command(&self, node_id: NodeId, new_pdr: f32) {
        match &self.drone_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(DroneCommand::SetPacketDropRate(new_pdr));
            }
            None => {
                log::error!("Specified node does not exist");
            }
        }
    }

    pub fn send_add_sender_command(&self, node_id: NodeId, node_to_id: NodeId) {
        // drone -> any
        let mut matched = match (
            &self.drone_command_channels.get(&node_id),
            &self.packet_channels.get(&node_to_id),
        ) {
            (Some(c1), Some(c2)) => {
                c1.send(DroneCommand::AddSender(node_to_id, c2.0.clone()));
                true
            }
            _ => false,
        };

        // client -> any
        if !matched {
            matched = match (
                &self.client_command_channels.get(&node_id),
                &self.packet_channels.get(&node_to_id),
            ) {
                (Some(c1), Some(c2)) => {
                    c1.send(ClientCommand::AddSender(node_to_id, c2.0.clone()));
                    true
                }
                _ => false,
            };
        }

        // server -> any
        if !matched {
            matched = match (
                &self.server_command_channels.get(&node_id),
                &self.packet_channels.get(&node_to_id),
            ) {
                (Some(c1), Some(c2)) => {
                    c1.send(ServerCommand::AddSender(node_to_id, c2.0.clone()));
                    true
                }
                _ => false,
            };
        }

        if !matched {
            log::error!("Tryind to add link to unexisting node");
        }
        {
            log::info!("Link added");
        }
    }

    pub fn send_remove_sender_command(&self, node_id: NodeId, node_to_id: NodeId) {
        // drone -> any
        let mut matched = match (
            &self.drone_command_channels.get(&node_id),
            &self.packet_channels.get(&node_to_id),
        ) {
            (Some(c1), c2) => {
                if c2.is_none() {
                    log::warn!("Trying to remove a channel from unexisting node");
                }
                c1.send(DroneCommand::RemoveSender(node_to_id));
                true
            }
            _ => false,
        };

        // client -> any
        if !matched {
            matched = match (
                &self.client_command_channels.get(&node_id),
                &self.packet_channels.get(&node_to_id),
            ) {
                (Some(c1), c2) => {
                    if c2.is_none() {
                        log::warn!("Trying to remove a channel from unexisting node");
                    }
                    c1.send(ClientCommand::RemoveSender(node_to_id));
                    true
                }
                _ => false,
            };
        }

        // server -> any
        if !matched {
            matched = match (
                &self.server_command_channels.get(&node_id),
                &self.packet_channels.get(&node_to_id),
            ) {
                (Some(c1), c2) => {
                    if c2.is_none() {
                        log::warn!("Trying to remove a channel from unexisting node");
                    }
                    c1.send(ServerCommand::RemoveSender(node_to_id));
                    true
                }
                _ => false,
            };
        }

        if !matched {
            log::error!("Tryind to remove link from unexisting node");
        }
        {
            log::info!("Link added");
        }
    }

    pub fn spawn_drone(
        &mut self,
        node_id: NodeId,
        pdr: f32,
        neighbors: &[NodeId],
        vendor: DroneVendor,
    ) {
        let drone_packet_channel = unbounded::<Packet>();
        let drone_event_channel = unbounded::<DroneEvent>();
        let drone_command_channel = unbounded::<DroneCommand>();

        self.packet_channels
            .insert(node_id, drone_packet_channel.clone());
        self.node_event_channels
            .insert(node_id, drone_event_channel.1);
        self.drone_command_channels
            .insert(node_id, drone_command_channel.0);

        let mut drone_packet_senders = HashMap::new();
        for neighbor in neighbors.iter() {
            // new drone -> neighbors
            drone_packet_senders.insert(
                *neighbor,
                self.packet_channels.get(neighbor).unwrap().0.clone(),
            );

            self.send_add_sender_command(*neighbor, node_id);
        }

        let barrier = Arc::new(Barrier::new(2));
        spawn_drone_thread_by_vendor(
            node_id,
            drone_event_channel.0,
            drone_command_channel.1,
            drone_packet_channel.1,
            drone_packet_senders,
            pdr,
            barrier.clone(),
            vendor,
        );
        barrier.wait();
    }

    pub fn handle_sc_shortcut(&self, packet: Packet) {
        match &packet.pack_type {
            PacketType::MsgFragment(fragment) => {
                log::error!("Techinally, msg fragments cannot be sent throug sc shortcuts")
            }
            PacketType::FloodRequest(flood_request) => {
                log::error!("Techinally, flood requests cannot be sent throug sc shortcuts")
            }
            _ => {}
        }

        match packet.routing_header.destination() {
            Some(last_hop) => match self.packet_channels.get(&last_hop) {
                Some(channel) => {
                    let mut new_packet = packet.clone();
                    new_packet.routing_header.hop_index = new_packet.routing_header.hops.len() - 1;
                    channel.0.send(new_packet);
                    println!("Sending shortcut to node {}", last_hop);
                    log::info!("Sending shortcut to node {}", last_hop)
                }
                None => {
                    log::error!("Shortcut destination does not exist")
                }
            },
            None => log::error!("Cannot get shortcut destination"),
        }
    }

    pub fn send_control_packet(&self, server_command: ServerCommand, node_id: NodeId) {
        match &self.server_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(server_command);
                log::info!("Sending to node {}", node_id)
            }
            None => {
                log::error!("Specified node does not exist")
            }
        }
    }
}
