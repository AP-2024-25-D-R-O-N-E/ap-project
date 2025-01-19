use crate::simulation_controller::{ClientEvent, SCEvent, ServerEvent};
use petgraph::graph::NodeIndex;
use wg_2024::{controller::DroneEvent, packet::Packet};

pub struct EventsState {
    pub events: Vec<SCEvent>,
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
}

impl EventsState {
    pub fn get_events_list(&self, display_options: DisplayOptions) -> Vec<SCEvent> {
        match display_options.specific_index {
            Some(specific_index) => self
                .events
                .clone()
                .into_iter()
                .filter(|e| match e.get_sender_node_index() {
                    Some(idx) => idx == specific_index,
                    None => false,
                })
                .collect(),
            None => self
                .events
                .clone()
                .into_iter()
                .filter(|e| match e {
                    SCEvent::ClientEvent(client_event) => display_options.clients,
                    SCEvent::ServerEvent(server_event) => display_options.servers,
                    SCEvent::DroneEvent(drone_event) => display_options.drones,
                })
                .collect(),
        }
    }

    pub fn add_with_limit(&mut self, event: SCEvent, limit: usize) {
        if self.events.len() > limit {
            self.events.remove(0);
        }
        self.events.push(event);
    }
}

impl Default for EventsState {
    fn default() -> Self {
        EventsState {
            events: vec![],
            // drone_events: vec![],
            // client_events: vec![],
            // server_events: vec![],
        }
    }
}

#[derive(Clone, Copy)]
pub struct DisplayOptions {
    pub drones: bool,
    pub clients: bool,
    pub servers: bool,
    pub specific_index: Option<wg_2024::network::NodeId>,
}

impl DisplayOptions {
    pub const ALL: DisplayOptions = DisplayOptions {
        drones: true,
        clients: true,
        servers: true,
        specific_index: None,
    };
    pub const DRONES_ONLY: DisplayOptions = DisplayOptions {
        drones: true,
        clients: false,
        servers: false,
        specific_index: None,
    };
    pub const CLIENTS_ONLY: DisplayOptions = DisplayOptions {
        drones: false,
        clients: true,
        servers: false,
        specific_index: None,
    };
    pub const SERVERS_ONLY: DisplayOptions = DisplayOptions {
        drones: false,
        clients: false,
        servers: true,
        specific_index: None,
    };
    pub fn from_index(node_index: wg_2024::network::NodeId) -> DisplayOptions {
        DisplayOptions {
            drones: true,
            clients: true,
            servers: true,
            specific_index: Some(node_index),
        }
    }
}
