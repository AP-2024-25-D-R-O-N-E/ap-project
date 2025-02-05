use wg_2024::{controller::DroneEvent, network::NodeId, packet::Packet};

/// From client to controller
#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to client
#[derive(Debug, Clone, serde::Serialize)]
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
pub enum SCEventType {
    ClientEvent(ClientEvent),
    ServerEvent(ServerEvent),
    DroneEvent(DroneEvent),
}

#[derive(Debug, Clone)]
pub struct SCEvent {
    pub event_type: SCEventType,
    pub sender_id: NodeId,
}

impl From<ClientEvent> for SCEventType {
    fn from(event: ClientEvent) -> Self {
        SCEventType::ClientEvent(event)
    }
}

impl From<ServerEvent> for SCEventType {
    fn from(event: ServerEvent) -> Self {
        SCEventType::ServerEvent(event)
    }
}

impl From<DroneEvent> for SCEventType {
    fn from(event: DroneEvent) -> Self {
        SCEventType::DroneEvent(event)
    }
}

impl SCEventType {
    pub fn get_sender_node_index(&self) -> Option<u8> {
        self.get_packet().routing_header.previous_hop()
    }

    pub fn get_packet(&self) -> Packet {
        let packet = match self {
            SCEventType::ClientEvent(client_event) => match client_event {
                ClientEvent::PacketSent(packet) => packet,
                ClientEvent::PacketDropped(packet) => packet,
            },
            SCEventType::ServerEvent(server_event) => match server_event {
                ServerEvent::PacketSent(packet) => packet,
                ServerEvent::PacketDropped(packet) => packet,
            },
            SCEventType::DroneEvent(drone_event) => match drone_event {
                DroneEvent::PacketSent(packet) => packet,
                DroneEvent::PacketDropped(packet) => packet,
                DroneEvent::ControllerShortcut(packet) => packet,
            },
        };

        packet.clone()
    }

    pub fn get_sender_type(&self) -> String {
        match self {
            SCEventType::ClientEvent(client_event) => "Client",
            SCEventType::ServerEvent(server_event) => "Server",
            SCEventType::DroneEvent(drone_event) => "Drone",
        }
        .to_string()
    }

    pub fn get_event_type(&self) -> String {
        // TODO implement unimplemented
        match self {
            SCEventType::ClientEvent(client_event) => "Unimplemented",
            SCEventType::ServerEvent(server_event) => "Unimplemented",
            SCEventType::DroneEvent(drone_event) => match drone_event {
                DroneEvent::PacketSent(packet) => "PacketSent",
                DroneEvent::PacketDropped(packet) => "PacketDropped",
                DroneEvent::ControllerShortcut(packet) => "ControllerShortcut",
            },
        }
        .to_string()
    }
}

impl SCEvent {
    pub fn get_sender_node_index(&self) -> Option<u8> {
        self.event_type.get_sender_node_index()
    }

    pub fn get_packet(&self) -> Packet {
        self.event_type.get_packet()
    }

    pub fn get_sender_type(&self) -> String {
        self.event_type.get_sender_type()
    }

    pub fn get_event_type(&self) -> String {
        self.event_type.get_event_type()
    }
}

impl SCEvent {
    /// Create a new SCEvent
    pub fn new(sender_id: NodeId, event_type: SCEventType) -> SCEvent {
        SCEvent {
            event_type,
            sender_id,
        }
    }
}

impl std::fmt::Display for SCEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_packet())
    }
}

impl std::fmt::Display for SCEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.sender_id, self.event_type)
    }
}
