use egui_graphs::Graph;
use std::collections::HashMap;

use crate::simulation_controller::SimulationController;

use super::{
    ConsoleSectionState, DebugSectionState, EventsState, GraphSectionState, ModifyTopologyState,
    NodeInfoSectionState, SettingsSectionState, TestSectionState, ToolbarSectionState,
};

pub struct State {
    pub test_section: TestSectionState,
    pub debug_section: DebugSectionState,
    pub settings_section: SettingsSectionState,
    pub graph_section: GraphSectionState,
    pub events: EventsState,
    pub toolbar_section: ToolbarSectionState,
    pub node_info_section: NodeInfoSectionState,
    pub console_section: ConsoleSectionState,
    pub modify_topology_section: ModifyTopologyState,
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
        let graph_section = GraphSectionState::default();

        let mut map = HashMap::new();
        for n in graph_section.g.g.node_indices() {
            map.insert(graph_section.g.node(n).unwrap().payload().wg_id, n);
        }

        let mut modify_topology_section = ModifyTopologyState::default();
        let mut max = 0;
        for (node_id, _) in &graph_section.node_id_map {
            if *node_id > max {
                max = *node_id;
            }
        }
        modify_topology_section.id = max + 1;

        Self {
            test_section: Default::default(),
            debug_section: Default::default(),
            settings_section: Default::default(),
            graph_section,
            events: Default::default(),
            toolbar_section: Default::default(),
            node_info_section: Default::default(),
            console_section: Default::default(),
            modify_topology_section,
        }
    }
}
