use egui::{CollapsingHeader, Color32, RichText, Ui};
use petgraph::algo::{self, dijkstra::dijkstra};
use wg_2024::network::NodeId;

use crate::simulation_controller::{state::State, SimulationController};

pub fn draw_section_testing(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    // CollapsingHeader::new("Last Events")
    //     .default_open(false)
    //     .show(ui, |ui| {
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default fragment").clicked() {
    //                 simulation_controller
    //                     .send_default_msg_fragment(state.test_section.send_default_fragment_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_fragment_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default flood request").clicked() {
    //                 simulation_controller.send_default_flood_request(
    //                     state.test_section.send_default_flood_request_node_id,
    //                 )
    //             }
    //             ui.add(
    //                 egui::DragValue::new(
    //                     &mut state.test_section.send_default_flood_request_node_id,
    //                 )
    //                 .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default ack").clicked() {
    //                 simulation_controller
    //                     .send_default_ack(state.test_section.send_default_ack_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_ack_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default nack").clicked() {
    //                 simulation_controller
    //                     .send_default_nack(state.test_section.send_default_nack_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_nack_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //     });

    CollapsingHeader::new("Send msg fragment")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    send_msg_fragment_section(ui, state, simulation_controller)
                });

            ui.horizontal(|ui| {
                if ui.button("Send default fragment").clicked() {
                    simulation_controller
                        .send_default_msg_fragment(state.test_section.send_default_fragment_node_id)
                }
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_fragment_node_id)
                        .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default flood request").clicked() {
                    simulation_controller.send_default_flood_request(
                        state.test_section.send_default_flood_request_node_id,
                    )
                }
                ui.add(
                    egui::DragValue::new(
                        &mut state.test_section.send_default_flood_request_node_id,
                    )
                    .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default ack").clicked() {
                    simulation_controller
                        .send_default_ack(state.test_section.send_default_ack_node_id)
                }
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_ack_node_id)
                        .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default nack").clicked() {
                    simulation_controller
                        .send_default_nack(state.test_section.send_default_nack_node_id)
                }
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_nack_node_id)
                        .speed(0.1),
                );
            });
        });
}

pub fn send_msg_fragment_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    // Routing Path Input
    ui.label("Routing path");
    if ui
        .add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut state.test_section.routing_path_string)
                .hint_text("Enter node IDs separated by commas"), // .tooltip_text("The list of node id separated by a comma"),
        )
        .changed()
    {}

    ui.end_row();

    // Msg data input
    ui.label("Msg data");
    if ui
        .add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut state.test_section.msg_frag_data_string)
                .hint_text("The msg_fragment payload. Just input a stream of max 128 characters"), // .tooltip_text(),
        )
        .changed()
    {}
    ui.end_row();

    // Get from nodes button
    if ui
        .button("Get from selection")
        .on_hover_text("Get a random path between the two selected nodes")
        .clicked()
    {
        if state.graph_section.g.selected_nodes().len() != 2 {
            state.test_section.status_flag = Some(Err("Please select exactly 2 nodes".to_string()));
        } else {
            state.test_section.status_flag = None;
            let start_node = state.graph_section.g.selected_nodes()[0].clone();
            let end_node = state.graph_section.g.selected_nodes()[1].clone();
            let g = &*state.graph_section.g.g();

            let path = algo::astar(g, start_node, |n| n == end_node, |e| 1, |_| 0);
            match path {
                Some((_, path)) => {
                    let mut new_path = String::new();
                    for n in path {
                        let actual_node_index =
                            state.graph_section.g.node(n).unwrap().payload().wg_id;
                        new_path.push_str(format!("{}, ", actual_node_index).as_str());
                    }
                    if new_path.len() != 0 {
                        new_path.truncate(new_path.len() - 2);
                    }
                    state.test_section.routing_path_string = new_path
                }
                None => todo!(),
            }
        }
    }

    // Send Button
    if ui
        .add_sized(ui.available_size(), egui::Button::new("Send"))
        .clicked()
    {
        let mut ok = true;
        for c in state.test_section.routing_path_string.chars() {
            if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
                state.test_section.status_flag = Some(Err("The path is malformed".to_string()));
                ok = false;
                break;
            }
        }

        for c in state.test_section.msg_frag_data_string.chars() {
            if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
                state.test_section.status_flag = Some(Err("The data is malformed".to_string()));
                ok = false;
                break;
            }
        }
        let mut parsed_data_vec: Vec<u8> = state
            .test_section
            .msg_frag_data_string
            .split(',')
            .filter_map(|s| {
                let rv = s.trim().parse::<u8>().ok();
                match rv {
                    None => {
                        state.test_section.status_flag =
                            Some(Err("The data is malformed".to_string()));
                        ok = false;
                    }
                    _ => (),
                }

                rv
            })
            .collect();

        if parsed_data_vec.len() > 128 {
            state.test_section.status_flag =
                Some(Err("Data should be at most 128 chars long".to_string()));
            ok = false;
        }
        parsed_data_vec.resize(128, 0);

        if ok {
            let parsed_path: Vec<NodeId> = state
                .test_section
                .routing_path_string
                .split(',')
                .filter_map(|s| s.trim().parse::<NodeId>().ok())
                .collect();
            let mut parsed_data: [u8; 128] = [0; 128];
            match parsed_data_vec.try_into() {
                Ok(v) => {
                    parsed_data = v;
                }
                Err(_) => (),
            }

            let mut packet = simulation_controller.default_msg_fragment.clone();
            packet.routing_header.hops = parsed_path;
            match &mut packet.pack_type {
                wg_2024::packet::PacketType::MsgFragment(fragment) => {
                    fragment.data = parsed_data;
                }
                _ => (),
            }
            simulation_controller.send_msg_fragment(packet);
        }
    }

    ui.end_row();
    match &state.test_section.status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(Color32::GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(Color32::DARK_RED));
            }
        },
        None => (),
    }
    ui.end_row();
}
