use crate::simulation_controller::{ClientEvent, ServerEvent};
use wg_2024::controller::DroneEvent;

pub struct EventsState {
    pub drone_events: Vec<DroneEvent>,
    pub client_events: Vec<ClientEvent>,
    pub server_events: Vec<ServerEvent>,
}

impl EventsState {}

impl Default for EventsState {
    fn default() -> Self {
        EventsState {
            drone_events: vec![],
            client_events: vec![],
            server_events: vec![],
        }
    }
}
