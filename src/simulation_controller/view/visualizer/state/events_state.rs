use crate::simulation_controller::{ClientEvent, SCEvent, ServerEvent};
use wg_2024::controller::DroneEvent;

pub struct EventsState {
    pub events: Vec<SCEvent>,
}

impl EventsState {
    pub fn get_events_list(&self, display_options: DisplayOptions) -> Vec<SCEvent> {
        // self.events.clone()
        self.events
            .clone()
            .into_iter()
            .filter(|e| match e {
                SCEvent::ClientEvent(client_event) => display_options.clients,
                SCEvent::ServerEvent(server_event) => display_options.servers,
                SCEvent::DroneEvent(drone_event) => display_options.drones,
            })
            .collect()
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
}

impl DisplayOptions {
    pub const ALL: DisplayOptions = DisplayOptions {
        drones: true,
        clients: true,
        servers: true,
    };
    pub const DRONES_ONLY: DisplayOptions = DisplayOptions {
        drones: true,
        clients: false,
        servers: false,
    };
    pub const CLIENTS_ONLY: DisplayOptions = DisplayOptions {
        drones: false,
        clients: true,
        servers: false,
    };
    pub const SERVERS_ONLY: DisplayOptions = DisplayOptions {
        drones: false,
        clients: false,
        servers: true,
    };
}
