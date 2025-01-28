use egui::Ui;
use egui_extras::TableRow;

use crate::simulation_controller::{state::State, SCEvent};

impl SCEvent {
    pub fn draw(&self, row: &mut TableRow, state: &mut State) {
        row.col(|ui| {
            ui.label(format!(
                "{} {}",
                self.get_sender_type(),
                self.get_sender_node_index()
                    .map_or("".to_string(), |v| v.to_string())
            ));
        });
        row.col(|ui| {
            ui.label(self.get_packet().session_id.to_string());
        });
        row.col(|ui| {
            ui.label(self.get_event_type());
        });

        row.col(|ui| {
            ui.horizontal(|ui| {});
        });
    }
}
