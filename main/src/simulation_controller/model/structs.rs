use std::path::PathBuf;

use crossbeam::channel::Sender;
use wg_2024::{
    controller::DroneEvent,
    network::NodeId,
    packet::{self, Packet},
};

use crate::fragmentation::message::ChatMessage;

/// From client to controller
#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketSent(Packet),
    PacketReceived(Packet),
    TextMessage {
        from: NodeId,
        to: NodeId,
        text: String,
    },

    CreatedFileLocal{
        from: NodeId,
        to: NodeId,
        file_path: PathBuf,
    },

    FileMessage {
        from: NodeId,
        to: NodeId,
        file_path: PathBuf,
    },

    ResponseClientsReceived(Vec<NodeId>),
    AcknolewdgedAsClient,
    ResponseHistoryReceived {
        partner: NodeId,
        history: Vec<ChatMessage>,
    },
    UnregisteredSenderError,
    UnregisteredRecipientError,
    UnsupportedMessageTypeError,
}

/// From controller to client
#[derive(Debug)]
pub enum ClientCommand {
    StartFlooding,

    AddSender(NodeId, Sender<Packet>),
    RemoveSender(NodeId),

    GetResponseClient,
    RegisterAsClient,
    UnregisterAsClient,
    OpenChatWith(NodeId),
    SendTextMessageTo {
        receiver: NodeId,
        message: String,
    },
    // the file message is only temporary and will be modified later
    SendFileMessageTo {
        receiver: NodeId,
        file_path: PathBuf,
    },
}

/// From server to controller
#[derive(Debug, Clone)]
pub enum ServerEvent {
    PacketSent(Packet),
    PacketReceived(Packet),
}

/// From controller to server
#[derive(Debug, Clone)]
pub enum ServerCommand {
    NetworkInitialized,
    AddSender(NodeId, Sender<Packet>),
    RemoveSender(NodeId),
}

/// Common interface for events
#[derive(Debug, Clone)]
pub enum SCEventType {
    Client(ClientEvent),
    Server(ServerEvent),
    Drone(DroneEvent),
}

#[derive(Debug, Clone)]
pub struct SCEvent {
    pub event_type: SCEventType,
    pub sender_id: NodeId,
}

impl From<ClientEvent> for SCEventType {
    fn from(event: ClientEvent) -> Self {
        SCEventType::Client(event)
    }
}

impl From<ServerEvent> for SCEventType {
    fn from(event: ServerEvent) -> Self {
        SCEventType::Server(event)
    }
}

impl From<DroneEvent> for SCEventType {
    fn from(event: DroneEvent) -> Self {
        SCEventType::Drone(event)
    }
}

impl SCEventType {
    pub fn get_sender_node_index(&self) -> Option<u8> {
        match self.get_packet() {
            Some(packet) => packet.routing_header.previous_hop(),
            None => None,
        }
    }

    pub fn get_packet(&self) -> Option<Packet> {
        match self {
            SCEventType::Client(client_event) => match client_event {
                ClientEvent::PacketSent(packet) => Some(packet.clone()),
                ClientEvent::PacketReceived(packet) => Some(packet.clone()),
                ClientEvent::TextMessage { .. } => None,
                ClientEvent::FileMessage { .. } => None,
                ClientEvent::ResponseClientsReceived(..) => None,
                ClientEvent::AcknolewdgedAsClient => None,
                ClientEvent::ResponseHistoryReceived { .. } => None,
                ClientEvent::UnregisteredSenderError => None,
                ClientEvent::UnregisteredRecipientError => None,
                ClientEvent::UnsupportedMessageTypeError => None,
                ClientEvent::CreatedFileLocal { from: _, to: _, file_path: _ } => None,
            },
            SCEventType::Server(server_event) => match server_event {
                ServerEvent::PacketSent(packet) => Some(packet.clone()),
                ServerEvent::PacketReceived(packet) => Some(packet.clone()),
            },
            SCEventType::Drone(drone_event) => match drone_event {
                DroneEvent::PacketSent(packet) => Some(packet.clone()),
                DroneEvent::PacketDropped(packet) => Some(packet.clone()),
                DroneEvent::ControllerShortcut(packet) => Some(packet.clone()),
            },
        }
    }

    pub fn get_sender_type(&self) -> String {
        match self {
            SCEventType::Client(_client_event) => "Client",
            SCEventType::Server(_server_event) => "Server",
            SCEventType::Drone(_drone_event) => "Drone",
        }
        .to_string()
    }

    pub fn get_event_type(&self) -> String {
        match self {
            SCEventType::Client(client_event) => match client_event {
                ClientEvent::PacketSent(_packet) => "PacketSent",
                ClientEvent::PacketReceived(_packet) => "PacketReceived",
                ClientEvent::TextMessage { .. } => "TextMessage",
                ClientEvent::FileMessage { .. } => "FileMessage",
                ClientEvent::ResponseClientsReceived(..) => "ResponseClientsReceived",
                ClientEvent::AcknolewdgedAsClient => "AcknolewdgedAsClient",
                ClientEvent::ResponseHistoryReceived { .. } => "ResponseHistoryReceived",
                ClientEvent::UnregisteredSenderError => "UnregisteredSenderError",
                ClientEvent::UnregisteredRecipientError => "UnregisteredRecipientError",
                ClientEvent::UnsupportedMessageTypeError => "UnsupportedMessageTypeError",
                ClientEvent::CreatedFileLocal { from: _, to: _, file_path: _ } => "Local File Created on Client",
            },
            SCEventType::Server(server_event) => match server_event {
                ServerEvent::PacketSent(_packet) => "PacketSent",
                ServerEvent::PacketReceived(_packet) => "PacketReceived",
            },
            SCEventType::Drone(drone_event) => match drone_event {
                DroneEvent::PacketSent(_packet) => "PacketSent",
                DroneEvent::PacketDropped(_packet) => "PacketDropped",
                DroneEvent::ControllerShortcut(_packet) => "ControllerShortcut",
            },
        }
        .to_string()
    }

    #[allow(dead_code)]
    pub fn get_packet_type(&self) -> Option<String> {
        match self.get_packet() {
            Some(packet) => Some(
                match packet.pack_type {
                    packet::PacketType::MsgFragment(_fragment) => "MsgFragment",
                    packet::PacketType::Ack(_ack) => "Ack",
                    packet::PacketType::Nack(_nack) => "Nack",
                    packet::PacketType::FloodRequest(_flood_request) => "FlooadRequest",
                    packet::PacketType::FloodResponse(_flood_response) => "FloodResponse",
                }
                .to_string(),
            ),

            None => None,
        }
    }
}

impl SCEvent {
    #[allow(dead_code)]
    pub fn get_sender_node_index(&self) -> Option<u8> {
        self.event_type.get_sender_node_index()
    }

    pub fn get_packet(&self) -> Option<Packet> {
        self.event_type.get_packet()
    }

    #[allow(dead_code)]
    pub fn get_sender_type(&self) -> String {
        self.event_type.get_sender_type()
    }

    #[allow(dead_code)]
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
        write!(f, "{:?}", self.get_packet())
    }
}

impl std::fmt::Display for SCEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.sender_id, self.event_type)
    }
}
