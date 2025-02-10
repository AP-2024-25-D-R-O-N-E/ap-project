use egui_file_dialog::FileDialog;
use egui_graphs::Graph;
use std::collections::HashMap;

use crate::simulation_controller::{node::UiNodeType, SimulationController};

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
    pub file_dialog: FileDialog,
}

impl State {
    pub fn from(sc: &SimulationController) -> State {
        for node in sc.topology.node_weights() {
            match &node.node_type {
                UiNodeType::Server(ui_server_node) => sc.send_server_start_flood(node.wg_id),
                UiNodeType::Client(ui_client_node) => sc.send_client_start_flood(node.wg_id),
                UiNodeType::Drone(ui_drone_node) => (),
            }
        }

        State {
            test_section: TestSectionState::default(),
            debug_section: DebugSectionState::default(),
            settings_section: SettingsSectionState::default(),
            graph_section: GraphSectionState::new(sc.topology.clone()),
            events: Default::default(),
            toolbar_section: Default::default(),
            node_info_section: NodeInfoSectionState::from(&sc.topology),
            console_section: Default::default(),
            modify_topology_section: ModifyTopologyState::from(&sc.topology),
            file_dialog: Default::default(),
        }
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
        for node_id in graph_section.node_id_map.keys() {
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
            file_dialog: Default::default(),
        }
    }
}
