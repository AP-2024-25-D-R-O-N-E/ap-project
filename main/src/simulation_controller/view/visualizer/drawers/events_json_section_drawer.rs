use crate::simulation_controller::{serialization_ref_structs::AsSerializable, SCEvent};
use egui::Ui;

use crate::simulation_controller::state::{events_state, State};

pub fn draw_all_events_as_json(ui: &mut Ui, state: &State) {
    for event in state
        .events
        .get_events_list(events_state::DisplayOptions::ALL)
    {
        let rv = serde_json::to_string_pretty(&event.as_serializable()).unwrap();
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        egui_extras::syntax_highlighting::code_view_ui(ui, &theme, rv.as_str(), "json");
        ui.separator();
    }
}
pub fn draw_event_as_json(ui: &mut Ui, event: &SCEvent) {
    let rv = serde_json::to_string_pretty(&event.as_serializable()).unwrap();
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
    egui_extras::syntax_highlighting::code_view_ui(ui, &theme, rv.as_str(), "json");
}
