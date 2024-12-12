use egui::{CollapsingHeader, Ui};

use crate::simulation_controller::{state::State, SimulationController};

pub fn draw_section_testing(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    CollapsingHeader::new("Last Events")
        .default_open(true)
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
}
