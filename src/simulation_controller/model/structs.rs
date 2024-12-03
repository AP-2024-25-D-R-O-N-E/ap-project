use wg_2024::packet::Packet;

/// From client to controller
#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to client
#[derive(Debug, Clone)]
pub enum ClientCommand {}

/// From server to controller
#[derive(Debug, Clone)]
pub enum ServerEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to server
#[derive(Debug, Clone)]
pub enum ServerCommand {}
