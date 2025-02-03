use egui::{CollapsingHeader, Context, RichText, ScrollArea, Ui, Window};
use egui_extras::{Size, StripBuilder};
use petgraph::graph::{EdgeIndex, NodeIndex};
use wg_2024::network::NodeId;

use crate::{
    initializer::drone_vendor::DroneVendor,
    simulation_controller::{
        node::{UiDroneNode, UiNodePayload, UiNodeType},
        state::{DisplayOptions, NodeInfoSectionState, State},
        util::{
            self, add_drone, check_drone_addition, check_node_removal, colors, parse_string,
            remove_node,
        },
        SimulationController,
    },
};

use crate::simulation_controller::util::{
    get_drone_node_from_state, get_payload_from_state, get_payload_mut_from_state,
};

pub fn draw_modify_topology_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    egui::Grid::new("my_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Pdr");
            ui.add(egui::Slider::new(
                &mut state.modify_topology_section.pdr,
                0.0..=1.0,
            ));

            ui.end_row();

            ui.label("Node id");
            ui.label(state.modify_topology_section.id.to_string());
            ui.end_row();

            ui.label("Neighbors");
            if ui
                .add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut state.modify_topology_section.neighbors)
                        .hint_text("Enter node IDs separated by commas"),
                )
                .changed()
            {}
            ui.end_row();

            ui.label("Drone vendor");
            egui::ComboBox::from_label("")
                .selected_text(format!(
                    "{}",
                    state.modify_topology_section.drone_vendor.to_string()
                ))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::MyDrone,
                        DroneVendor::MyDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::RustafarianDrone,
                        DroneVendor::RustafarianDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::LockheedRustin,
                        DroneVendor::LockheedRustin.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::RustyDrone,
                        DroneVendor::RustyDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::RustBustersDrone,
                        DroneVendor::RustBustersDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::CppEnjoyersDrone,
                        DroneVendor::CppEnjoyersDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::RustezeDrone,
                        DroneVendor::RustezeDrone.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::GetDroned,
                        DroneVendor::GetDroned.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::RustRoveri,
                        DroneVendor::RustRoveri.to_string(),
                    );
                    ui.selectable_value(
                        &mut state.modify_topology_section.drone_vendor,
                        DroneVendor::MyDrone,
                        DroneVendor::MyDrone.to_string(),
                    );
                });
        });

    if ui.button("Spawn").clicked() {
        if can_insert_drone(state) {
            insert_drone(state)
        }
    }

    match &state.modify_topology_section.status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new("Drone spawned with success").color(colors::MUTED_GREEN));
            }
            Err(error) => {
                ui.label(RichText::new(error).color(colors::MUTED_RED));
            }
        },
        None => (),
    }
}

fn can_insert_drone(state: &mut State) -> bool {
    match parse_string::<NodeId>(state.modify_topology_section.neighbors.clone()) {
        Ok(wg_neighbor_ids) => {
            match check_drone_addition(&mut state.graph_section, &wg_neighbor_ids) {
                Ok(_) => {
                    state.modify_topology_section.status_flag =
                        Some(Ok("Drone inserted".to_string()));
                    return true;
                }
                Err(status) => {
                    state.modify_topology_section.status_flag = Some(Err(status));
                    return false;
                }
            }
        }
        Err(error) => {
            state.modify_topology_section.status_flag = Some(Err(error));
            return false;
        }
    }
}

fn insert_drone(state: &mut State) {
    let neighbors =
        parse_string::<NodeId>(state.modify_topology_section.neighbors.clone()).unwrap();
    add_drone(
        &mut state.graph_section,
        &neighbors,
        state.modify_topology_section.id,
        state.modify_topology_section.pdr,
        state.modify_topology_section.drone_vendor.clone(),
    );
}

// fn spawn_drone(state: &mut State) {
//
// }
// pub fn draw_spawn_node_window(ctx: &Context, state: &mut State) {}
