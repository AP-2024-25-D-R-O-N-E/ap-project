use egui_graphs::Graph;

use crate::simulation_controller::SimulationController;

use super::{
    DebugSectionState, EventsState, GraphSectionState, SettingsSectionState, TestSectionState,
};

pub struct State {
    pub test_section: TestSectionState,
    pub debug_section: DebugSectionState,
    pub settings_section: SettingsSectionState,
    pub graph_section: GraphSectionState,
    pub events: EventsState,
}

impl State {
    pub fn from(sc: &SimulationController) -> State {
        let mut default_state = State::default();
        default_state.graph_section.g = Graph::from(&sc.topology);
        default_state
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            test_section: Default::default(),
            debug_section: Default::default(),
            settings_section: Default::default(),
            graph_section: Default::default(),
            events: Default::default(),
        }
    }
}
