use wg_2024::{
    network::NodeId,
    packet::{Packet, PacketType},
};

use super::simulation_controller::SimulationController;

impl SimulationController {
    pub fn send_default_msg_fragment(&self, node_id: NodeId) {
        self.send_msg_fragment(self.default_msg_fragment.clone(), node_id);
    }

    pub fn send_msg_fragment(&self, packet: Packet, node_id: NodeId) {
        match &self.packet_channels.get(&node_id) {
            Some(channel) => match &packet.pack_type {
                PacketType::MsgFragment(_) => {
                    channel.0.send(packet);
                    log::info!("Sending to node {}", node_id)
                }
                _ => {
                    log::error!("Provided package is not a msg fragment")
                }
            },
            None => {
                log::error!("Specified node does not exist")
            }
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
        self.send_ack(self.default_ack.clone(), node_id);
    }

    pub fn send_ack(&self, packet: Packet, node_id: NodeId) {
        match &self.packet_channels.get(&node_id) {
            Some(channel) => match &packet.pack_type {
                PacketType::Ack(_) => {
                    channel.0.send(packet);
                    log::info!("Sending to node {}", node_id)
                }
                _ => {
                    log::error!("Provided package is not an ack")
                }
            },
            None => {
                log::error!("Specified node does not exist")
            }
        }
    }
}
