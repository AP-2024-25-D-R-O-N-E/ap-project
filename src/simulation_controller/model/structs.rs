use std::fs::File;

use crossbeam::channel::Sender;
use egui::accesskit::Node;
use wg_2024::{controller::DroneEvent, network::NodeId, packet::Packet};

use crate::fragmentation::message::ChatMessage;

/// From client to controller
#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
    
    ResponseClientsReceived(Vec<NodeId>),
    AcknolewdgedAsClient,
    ResponseHistoryReceived{
        partner: NodeId,
        history: Vec<ChatMessage>
    },
    UnregisteredSenderError,
    UnregisteredRecipientError,
    UnsupportedMessageTypeError,
    TextMessage{
        from: NodeId,
        to: NodeId,
        text: String
    },
    FileMessage {
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: String,
    },
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
    SendTextMessageTo{
        receiver: NodeId,
        message: String,
    },
    SendFileMessageTo{
        receiver: NodeId,
        file: File
    }
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
        self.get_packet().routing_header.previous_hop()
    }

    pub fn get_packet(&self) -> Packet {
        let packet = match self {
            SCEvent::ClientEvent(client_event) => match client_event {
                ClientEvent::PacketSent(packet) => packet,
                ClientEvent::PacketDropped(packet) => packet,
                ClientEvent::ResponseClientsReceived(items) => todo!(),
                ClientEvent::AcknolewdgedAsClient => todo!(),
                ClientEvent::ResponseHistoryReceived { partner, history } => todo!(),
                ClientEvent::UnregisteredSenderError => todo!(),
                ClientEvent::UnregisteredRecipientError => todo!(),
                ClientEvent::UnsupportedMessageTypeError => todo!(),
                ClientEvent::TextMessage { from, to, text } => todo!(),
                ClientEvent::FileMessage { from, to, file, file_name } => todo!(),
                
            },
            SCEvent::ServerEvent(server_event) => match server_event {
                ServerEvent::PacketSent(packet) => packet,
                ServerEvent::PacketReceived(packet) => packet,
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

impl std::fmt::Display for SCEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_packet())
    }
}
