use wg_2024::{controller::DroneEvent, packet::Packet};

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

/// Common interface for events
#[derive(Debug, Clone)]
pub enum SCEvent {
    ClientEvent(ClientEvent),
    ServerEvent(ServerEvent),
    DroneEvent(DroneEvent),
}
impl From<ClientEvent> for SCEvent {
    fn from(event: ClientEvent) -> Self {
        SCEvent::ClientEvent(event)
    }
}

impl From<ServerEvent> for SCEvent {
    fn from(event: ServerEvent) -> Self {
        SCEvent::ServerEvent(event)
    }
}

impl From<DroneEvent> for SCEvent {
    fn from(event: DroneEvent) -> Self {
        SCEvent::DroneEvent(event)
    }
}

