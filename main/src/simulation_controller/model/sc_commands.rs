use std::{
    collections::HashMap,
    sync::{Arc, Barrier},
};

use crossbeam::channel::{unbounded, Sender};
use wg_2024::{
    controller::*,
    network::NodeId,
    packet::{Packet, PacketType},
};

use crate::initializer::{
    drone_vendor::DroneVendor,
    network_initializer::{spawn_drone_thread, spawn_drone_thread_by_vendor},
};

use super::simulation_controller::SimulationController;

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
            None => todo!(),
        }
    }

    pub fn send_default_flood_request(&self, node_id: NodeId) {
        self.send_flood_request(self.default_flood.clone(), node_id);
    }

    pub fn send_flood_request(&self, packet: Packet, node_id: NodeId) {
        match &self.packet_channels.get(&node_id) {
            Some(channel) => match &packet.pack_type {
                PacketType::FloodRequest(_) => {
                    channel.0.send(packet);
                    log::info!("Sending to node {}", node_id)
                }
                _ => {
                    log::error!("Provided package is not a flood request")
                }
            },
            None => {
                log::error!("Specified node does not exist")
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
            None => todo!(),
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

            None => todo!(),
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

    pub fn send_remove_sender_command(&self, node_id: NodeId, node_to_id: NodeId) {
        match &self.drone_command_channels.get(&node_id) {
            Some(command_channel) => {
                if !self.packet_channels.contains_key(&node_to_id) {
                    log::warn!("Removing a channel to unexisting node");
                } else {
                    command_channel.send(DroneCommand::RemoveSender(node_to_id));
                }
            }
            None => {
                log::error!("Specified node does not exist");
            }
        }
    }

    pub fn send_add_sender_command(&self, node_id: NodeId, node_to_id: NodeId) {
        match &self.drone_command_channels.get(&node_id) {
            Some(command_channel) => match self.packet_channels.get(&node_to_id) {
                Some(packet_channel) => {
                    command_channel.send(DroneCommand::AddSender(
                        node_to_id,
                        packet_channel.0.clone(),
                    ));
                }
                None => {
                    log::warn!("Trying to add a channel to unexisting node");
                }
            },
            None => {
                log::error!("Specified node does not exist");
            }
        }
    }

    pub fn spawn_drone(
        &mut self,
        node_id: NodeId,
        pdr: f32,
        neighbors: &Vec<NodeId>,
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
                self.packet_channels.get(&neighbor).unwrap().0.clone(),
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
}
