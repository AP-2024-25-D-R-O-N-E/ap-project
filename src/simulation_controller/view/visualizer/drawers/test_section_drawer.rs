use egui::{CollapsingHeader, Color32, RichText, Ui};
use petgraph::algo::

use crate::simulation_controller::{state::State, SimulationController};

pub fn draw_section_testing(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    CollapsingHeader::new("Last Events")
        .default_open(false)
        .show(ui, |ui| {
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
    CollapsingHeader::new("Send msg fragment")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| send_msg_fragment_section(ui, state))
        });
}

pub fn send_msg_fragment_section(ui: &mut Ui, state: &mut State) {
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
            state.test_section.status_flag = None
                let g = state.graph_section.g.g();
        }
    }

    // Send Button
    if ui
        .add_sized(ui.available_size(), egui::Button::new("Send"))
        .clicked()
    {}
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
