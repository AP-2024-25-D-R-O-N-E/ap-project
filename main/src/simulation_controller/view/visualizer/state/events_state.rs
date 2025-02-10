use crate::simulation_controller::{SCEvent, SCEventType};

#[derive(Default)]
pub struct EventsState {
    pub events: Vec<SCEvent>,
}

impl EventsState {
    pub fn get_events_list(&self, display_options: DisplayOptions) -> Vec<SCEvent> {
        match display_options.specific_index {
            Some(specific_index) => self
                .events
                .clone()
                .into_iter()
                .filter(|e| e.sender_id == specific_index)
                .collect(),
            None => self
                .events
                .clone()
                .into_iter()
                .filter(|e| match &e.event_type {
                    SCEventType::Client(client_event) => display_options.clients,
                    SCEventType::Server(server_event) => display_options.servers,
                    SCEventType::Drone(drone_event) => display_options.drones,
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
