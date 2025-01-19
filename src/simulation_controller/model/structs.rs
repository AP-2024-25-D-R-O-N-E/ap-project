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

impl SCEvent {
    pub fn get_sender_node_index(&self) -> Option<u8> {
        self.get_packet().routing_header.current_hop()
    }

    pub fn get_packet(&self) -> Packet {
        let packet = match self {
            SCEvent::ClientEvent(client_event) => match client_event {
                ClientEvent::PacketSent(packet) => packet,
                ClientEvent::PacketDropped(packet) => packet,
            },
            SCEvent::ServerEvent(server_event) => match server_event {
                ServerEvent::PacketSent(packet) => packet,
                ServerEvent::PacketDropped(packet) => packet,
            },
            SCEvent::DroneEvent(drone_event) => match drone_event {
                DroneEvent::PacketSent(packet) => packet,
                DroneEvent::PacketDropped(packet) => packet,
                DroneEvent::ControllerShortcut(packet) => packet,
            },
        };

        packet.clone()
    }
    pub fn get_sender_type(&self) -> String {
        match self {
            SCEvent::ClientEvent(client_event) => "Client",
            SCEvent::ServerEvent(server_event) => "Server",
            SCEvent::DroneEvent(drone_event) => "Drone",
        }
        .to_string()
    }
    pub fn get_event_type(&self) -> String {
        // TODO implement unimplemented
        match self {
            SCEvent::ClientEvent(client_event) => "Unimplemented",
            SCEvent::ServerEvent(server_event) => "Unimplemented",
            SCEvent::DroneEvent(drone_event) => match drone_event {
                DroneEvent::PacketSent(packet) => "PacketSent",
                DroneEvent::PacketDropped(packet) => "PacketDropped",
                DroneEvent::ControllerShortcut(packet) => "ControllerShortcut",
            },
        }
        .to_string()
    }
}
