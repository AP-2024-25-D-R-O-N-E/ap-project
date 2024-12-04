use egui::{CollapsingHeader, Ui};

use crate::simulation_controller::state::State;

pub fn draw_section_testing(ui: &mut Ui, state: &mut State) {
    CollapsingHeader::new("Last Events")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Send default fragment").clicked() {}
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_fragment_node_id)
                        .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default flood request").clicked() {}
                ui.add(
                    egui::DragValue::new(
                        &mut state.test_section.send_default_flood_request_node_id,
                    )
                    .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default ack").clicked() {}
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_ack_node_id)
                        .speed(0.1),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Send default nack").clicked() {}
                ui.add(
                    egui::DragValue::new(&mut state.test_section.send_default_nack_node_id)
                        .speed(0.1),
                );
            });
        });
}
