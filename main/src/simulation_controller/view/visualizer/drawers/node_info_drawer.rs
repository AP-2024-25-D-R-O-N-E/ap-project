use egui::{CollapsingHeader, Context, RichText, ScrollArea, Ui, Window};
use egui_extras::{Column, TableBuilder};
use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::simulation_controller::{
    node::{UiDroneNode, UiNodePayload, UiNodeType},
    state::{ClientState, DisplayOptions, DroneState, NodeInfoSectionState, ServerState, State},
    util::{self, check_node_removal, colors, remove_node},
    SimulationController,
};

use crate::simulation_controller::util::{
    get_drone_node_from_state, get_payload_from_state, get_payload_mut_from_state,
};

pub fn draw_infos_for_selected_nodes(
    ctx: &Context,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    for node_index in state.node_info_section.opened_windows.clone().iter() {
        draw_node_info(ctx, *node_index, state, simulation_controller);
    }
}

pub fn draw_node_info(
    ctx: &Context,
    node_index: NodeIndex,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    // let node_payload = state.graph_section.g.node(node_index).unwrap().payload();

    Window::new(format!(
        "Node {}",
        state
            .graph_section
            .g
            .node(node_index)
            .unwrap()
            .payload()
            .wg_id
            .clone()
    ))
    .collapsible(true)
    .resizable(true)
    .default_width(200.0)
    .default_height(100.0)
    .show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            ui.expand_to_include_rect(ui.available_rect_before_wrap());
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let payload = state.graph_section.g.node(node_index).unwrap().payload();
                    ui.label("Node type".to_string());
                    ui.horizontal(|ui| {
                        ui.label(payload.get_type().to_string());
                        ui.add_sized(ui.available_size(), egui::Label::new("".to_string()));
                    });
                    ui.end_row();

                    ui.label("Vendor".to_string());
                    ui.label(payload.vendor.to_string());

                    ui.end_row();

                    if ui.button("Close").clicked() {
                        println!(
                            "closing {:?} of {:?}",
                            node_index, state.node_info_section.opened_windows
                        );
                        state.node_info_section.opened_windows.remove(&node_index);
                    }
                    if ui.button("Close others").clicked() {
                        state
                            .node_info_section
                            .opened_windows
                            .retain(|opened_index| *opened_index == node_index)
                    }
                    ui.end_row();
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        // ui.separator();
                        ui.spacing();
                    });
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        // ui.separator();
                        ui.spacing();
                    });
                    ui.end_row();

                    if let Some(graph_node) = state.graph_section.g.node_mut(node_index) {
                        let ui_node = graph_node.payload_mut();
                        match &ui_node.node_type {
                            UiNodeType::Server(ui_server_node) => {
                                draw_server_specific(ui, node_index, state, simulation_controller);
                            }
                            UiNodeType::Client(ui_client_node) => {
                                draw_client_specific(ui, node_index, state, simulation_controller);
                            }
                            UiNodeType::Drone(ui_drone_node) => {
                                draw_drone_specific(ui, node_index, state, simulation_controller);
                            }
                        }
                    }
                });

            CollapsingHeader::new("Logs")
                .default_open(true)
                .show(ui, |ui| {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            //TODO draw node events
                            // ui.label(egui::RichText::new("This is red text!").color(egui::Color32::LIGHT_GRAY));
                            let events = state.events.get_events_list(DisplayOptions::ALL);
                            let text_height = ui.text_style_height(&egui::TextStyle::Body);
                            let payload = state
                                .graph_section
                                .g
                                .node(node_index)
                                .unwrap()
                                .payload()
                                .clone();

                            TableBuilder::new(ui)
                                .column(Column::auto().resizable(true))
                                .column(Column::auto().resizable(true))
                                .column(Column::auto().resizable(true))
                                .column(Column::remainder())
                                .striped(true)
                                .header(text_height, |mut header| {
                                    header.col(|ui| {
                                        ui.label("Sender");
                                    });
                                    header.col(|ui| {
                                        ui.label("Type");
                                    });
                                    header.col(|ui| {
                                        ui.label("Session id");
                                    });
                                    header.col(|ui| {
                                        ui.label("Other infos");
                                    });
                                })
                                .body(|mut body| {
                                    for (index, event) in state
                                        .events
                                        .get_events_list(DisplayOptions::from_index(payload.wg_id))
                                        .iter()
                                        .enumerate()
                                    {
                                        body.row(30.0, |mut row| {
                                            event.draw(&mut row, state);
                                        });
                                    }
                                });

                            // let mut table = TableBuilder::new(ui)
                            //     .striped(self.striped)
                            //     .resizable(self.resizable)
                            //     .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            //     .column(Column::auto())
                            //     .column(
                            //         Column::remainder()
                            //             .at_least(40.0)
                            //             .clip(true)
                            //             .resizable(true),
                            //     )
                            //     .column(Column::auto())
                            //     .column(Column::remainder())
                            //     .column(Column::remainder())
                            //     .min_scrolled_height(0.0)
                            //     .max_scroll_height(available_height);
                            //
                            // egui::Grid::new("my_grid")
                            //     .num_columns(2)
                            //     .spacing([40.0, 4.0])
                            //     .striped(true)
                            //     .show(ui, |ui| {
                            //         for event in state.events.get_events_list(DisplayOptions::ALL) {
                            //             // ui.label(egui::RichText::new(format!("{:?}", event)).color(egui::Color32::LIGHT_GRAY));
                            //             event.draw(ui, state)
                            //         }
                            //     });

                            // ui.label(
                            //     egui::RichText::new("This is green bold text!")
                            //         .color(egui::Color32::LIGHT_GRAY)
                            //         .strong(),
                            // );
                            //
                            // ui.label(
                            //     egui::RichText::new("This is blue italic text!")
                            //         .color(egui::Color32::LIGHT_GRAY)
                            //         .italics(),
                            // );
                        });

                    // ScrollArea::vertical().show(ui, |ui| {
                    //     for x in state.events.get_events_list(DisplayOptions::from_index(
                    //         state.graph_section.wg_id(node_index).unwrap(),
                    //     )) {
                    //         ui.label(
                    //             egui::RichText::new(format!("{:?}", x))
                    //                 .color(egui::Color32::LIGHT_GRAY),
                    //         );
                    //     }
                    // })
                });
        });
    });
}

fn draw_drone_specific(
    ui: &mut Ui,
    node_index: NodeIndex,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    let curr_node_wg_id = get_payload_from_state(state, node_index).unwrap().wg_id;

    if ui.button("Crash").clicked() {
        match check_node_removal(&mut state.graph_section.g, node_index) {
            Ok(_) => {
                let status_flag =
                    &mut get_drone_node_state(&mut state.node_info_section, node_index)
                        .crash_status_flag;
                remove_node(
                    &mut state.graph_section.g,
                    status_flag,
                    node_index,
                    &simulation_controller,
                )
            }
            Err(err) => {
                get_drone_node_state(&mut state.node_info_section, node_index).crash_status_flag =
                    Some(Err(err))
            }
        }
    }
    if get_drone_node_from_state(state, node_index).crashed {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Crashed".to_string()).color(util::colors::MUTED_RED));
            ui.add_sized(ui.available_size(), egui::Label::new("".to_string()));
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Running".to_string()).color(util::colors::MUTED_GREEN));
            ui.add_sized(ui.available_size(), egui::Label::new("".to_string()));
        });
    }
    ui.end_row();
    match &get_drone_node_state(&mut state.node_info_section, node_index).crash_status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        },
        None => (),
    };

    ui.end_row();

    ui.add(
        egui::Slider::new(
            &mut get_drone_node_from_state(state, node_index).pdr,
            0.0..=1.0,
        )
        .show_value(false),
    )
    .changed();
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut get_drone_node_from_state(state, node_index).pdr)
                .range(0.0..=1.0)
                .speed(0.01),
        );

        if ui
            .add_enabled(
                get_drone_node_from_state(state, node_index).last_committed_pdr
                    != get_drone_node_from_state(state, node_index).pdr,
                egui::Button::new("Update"),
            )
            .clicked()
        {
            get_drone_node_from_state(state, node_index).last_committed_pdr =
                get_drone_node_from_state(state, node_index).pdr;
            simulation_controller.send_set_pdr_command(
                get_payload_mut_from_state(state, node_index).unwrap().wg_id,
                get_drone_node_from_state(state, node_index).pdr,
            );
        };
    });
}

fn draw_server_specific(
    ui: &mut Ui,
    node_index: NodeIndex,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
}

fn draw_client_specific(
    ui: &mut Ui,
    node_index: NodeIndex,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    let curr_node_wg_id = get_payload_from_state(state, node_index).unwrap().wg_id;
    ui.horizontal(|ui| {
        if ui.button("Register".to_string()).clicked() {
            simulation_controller.register(curr_node_wg_id)
        }
        if ui.button("Unregister".to_string()).clicked() {
            simulation_controller.unregister(curr_node_wg_id)
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Start flood".to_string()).clicked() {
            simulation_controller.send_start_flood(curr_node_wg_id);
            // let curr_peer = get_client_node_state(&mut state.node_info_section, node_index).current_peer);
            // match curr_peer {
            //     Some(curr_peer) => simulation_controller.register(curr_peer);
            //     None => todo!(),
            // }
        }
        if ui.button("Get peers".to_string()).clicked() {
            simulation_controller.send_get_peers(curr_node_wg_id);
        }
    });
    ui.end_row();
    ui.label("Available peers");

    ui.label(format!(
        "{:?}",
        get_client_node_state(&mut state.node_info_section, node_index).available_peers
    ));
}

fn get_drone_node_state(
    state: &mut NodeInfoSectionState,
    node_index: NodeIndex,
) -> &mut DroneState {
    let x = state.states.get_mut(&node_index).unwrap();
    x.try_into().expect("Expected drone")
}

fn get_client_node_state(
    state: &mut NodeInfoSectionState,
    node_index: NodeIndex,
) -> &mut ClientState {
    let x = state.states.get_mut(&node_index).unwrap();
    x.try_into().expect("Expected client")
}

fn get_server_node_state(
    state: &mut NodeInfoSectionState,
    node_index: NodeIndex,
) -> &mut ServerState {
    let x = state.states.get_mut(&node_index).unwrap();
    x.try_into().expect("Expected server")
}
