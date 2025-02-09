use super::super::util::graph::*;
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
    simulation_controller: &mut SimulationController,
) {
    CollapsingHeader::new("Spawn drone")
        .default_open(true)
        .show(ui, |ui| {
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
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut state.modify_topology_section.neighbors)
                            .hint_text("Enter node IDs separated by commas"),
                    )
                    .changed();
                    ui.end_row();

                    ui.label("Drone vendor");
                    egui::ComboBox::from_label("")
                        .selected_text(state.modify_topology_section.drone_vendor.to_string())
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
                        });
                });

            if ui.button("Spawn").clicked() && can_insert_drone(state) {
                for (index, node_content) in state.graph_section.g.nodes_iter() {
                    let payload = node_content.payload();
                    match &payload.node_type {
                        UiNodeType::Server(ui_server_node) => {
                            simulation_controller.send_server_start_flood(payload.wg_id)
                        }
                        UiNodeType::Client(ui_client_node) => {
                            simulation_controller.send_client_start_flood(payload.wg_id)
                        }
                        UiNodeType::Drone(ui_drone_node) => (),
                    }
                }
                insert_drone(state, simulation_controller)
            }

            if let Some(status) = &state.modify_topology_section.status_flag {
                match status {
                    Ok(s) => {
                        ui.label(
                            RichText::new("Drone spawned with success").color(colors::MUTED_GREEN),
                        );
                    }
                    Err(error) => {
                        ui.label(RichText::new(error).color(colors::MUTED_RED));
                    }
                }
            }
        });

    CollapsingHeader::new("Add/remove sender")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    add_remove_sender_section(ui, state, simulation_controller)
                });
        });
}

fn can_insert_drone(state: &mut State) -> bool {
    match parse_string::<NodeId>(state.modify_topology_section.neighbors.clone()) {
        Ok(wg_neighbor_ids) => {
            match check_drone_addition(&mut state.graph_section, &wg_neighbor_ids) {
                Ok(_) => {
                    state.modify_topology_section.status_flag =
                        Some(Ok("Drone inserted".to_string()));
                    true
                }
                Err(status) => {
                    state.modify_topology_section.status_flag = Some(Err(status));
                    false
                }
            }
        }
        Err(error) => {
            state.modify_topology_section.status_flag = Some(Err(error));
            false
        }
    }
}

fn insert_drone(state: &mut State, simulation_controller: &mut SimulationController) {
    let neighbors =
        parse_string::<NodeId>(state.modify_topology_section.neighbors.clone()).unwrap();
    add_drone(
        &mut state.graph_section,
        &neighbors,
        state.modify_topology_section.id,
        state.modify_topology_section.pdr,
        state.modify_topology_section.drone_vendor,
        simulation_controller,
    );

    let mut first_free_id = state.modify_topology_section.id + 1;
    while (state.graph_section.node_id_map.contains_key(&first_free_id)) {
        first_free_id += 1;
    }

    state.modify_topology_section.id = first_free_id;
}

pub fn add_remove_sender_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    let check_parameters = |state: &mut State| {
        if state.graph_section.g.selected_nodes().len() != 2 {
            state.test_section.channel_modifier_status_flag =
                Some(Err("Please select exactly 2 nodes".to_string()));
            false
        } else {
            state.test_section.channel_modifier_status_flag = None;
            true
        }
    };

    if ui.button("Add sender").clicked() && check_parameters(state) {
        let node1 = state.graph_section.g.selected_nodes()[0];
        let node2 = state.graph_section.g.selected_nodes()[1];

        if check_edge_addition(
            &mut state.graph_section.g,
            &mut state.test_section.channel_modifier_status_flag,
            node1,
            node2,
        ) {
            add_edge_between(
                &mut state.graph_section.g,
                &mut state.test_section.channel_modifier_status_flag,
                node1,
                node2,
                simulation_controller,
            );
        }
    }

    if (ui.button("Remove sender").clicked()) && check_parameters(state) {
        let node1 = state.graph_section.g.selected_nodes()[0];
        let node2 = state.graph_section.g.selected_nodes()[1];

        if (check_edge_removal(
            &mut state.graph_section.g,
            &mut state.test_section.channel_modifier_status_flag,
            node1,
            node2,
        )) {
            remove_edges_between(
                &mut state.graph_section.g,
                &mut state.test_section.channel_modifier_status_flag,
                node1,
                node2,
                simulation_controller,
            );
        }
    }

    ui.end_row();

    if let Some(status) = &state.test_section.channel_modifier_status_flag {
        match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        }
    }
}
// fn spawn_drone(state: &mut State) {
//
// }
// pub fn draw_spawn_node_window(ctx: &Context, state: &mut State) {}
