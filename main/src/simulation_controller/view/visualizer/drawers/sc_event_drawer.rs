use egui::Ui;
use egui_extras::TableRow;

use crate::simulation_controller::{
    serialization_ref_structs::IntoSerializable, state::State, SCEvent, SCEventType,
};

use super::{draw_all_events_as_json, draw_event_as_json};

impl SCEvent {
    pub fn draw(&self, row: &mut TableRow, state: &mut State) {
        row.col(|ui| {
            ui.label(format!(
                "{} {}",
                self.event_type.get_sender_type(),
                self.sender_id
            ));
        });

        row.col(|ui| {
            ui.label(self.event_type.get_event_type());
        });

        row.col(|ui| {
            ui.label(self.get_packet().session_id.to_string());
        });

        row.col(|ui| {
            // let json_string = serde_json::to_string_pretty(&self).unwrap();
            // ui.label(format!("{:?}", self));
            // ui.horizontal(|ui| {
            // });
            //
            // if ui.button("?").clicked() {
            //     ui.to
            // }
            //     .on_hover_ui(|ui| {
            //     draw_event_as_json(ui, self);
            // });
        })
        .1
        .on_hover_ui(|ui| {
                draw_event_as_json(ui, self);
        });
    }
}
