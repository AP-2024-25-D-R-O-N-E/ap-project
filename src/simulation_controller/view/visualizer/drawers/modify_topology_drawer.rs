use egui::{CollapsingHeader, Context, RichText, ScrollArea, Ui, Window};
use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::simulation_controller::{
    node::{UiDroneNode, UiNodePayload, UiNodeType},
    state::{DisplayOptions, NodeInfoSectionState, State},
    util::{self, check_node_removal, colors, remove_node},
    SimulationController,
};

use crate::simulation_controller::util::{
    get_drone_node_from_state, get_payload_from_state, get_payload_mut_from_state,
};

pub fn draw_modify_topology_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    ui.label("prova");
}
// pub fn draw_spawn_node_window(ctx: &Context, state: &mut State) {}
