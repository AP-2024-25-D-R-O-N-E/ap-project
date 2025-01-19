use egui::Ui;

use crate::simulation_controller::{state::State, SCEvent};

impl SCEvent {
    pub fn draw(&self, ui: &mut Ui, state: &mut State) {
        ui.label(self.get_sender_type());
        ui.label(self.get_packet().session_id.to_string());
        ui.label(self.get_event_type());

        ui.horizontal(|ui| {});

        ui.end_row();
    }
}

// pub fn draw_sc_event(ui: &mut Ui, state: &mut State, event: SCEvent) {
//      ui.label(event.get_packet().);
//
// }
