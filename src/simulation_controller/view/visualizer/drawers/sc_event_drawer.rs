use egui::Ui;
use egui_extras::TableRow;

use crate::simulation_controller::{state::State, SCEvent};

impl SCEvent {
    pub fn draw(&self, mut row: TableRow, state: &mut State) {
        row.col(|ui| {
            ui.label(self.get_sender_type());
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

// pub fn draw_sc_event(ui: &mut Ui, state: &mut State, event: SCEvent) {
//      ui.label(event.get_packet().);
//
// }
