use crate::simulation_controller::serialization_ref_structs::IntoSerializable;
use egui::Ui;
use egui_extras::syntax_highlighting::CodeTheme;

use crate::simulation_controller::state::{events_state, State};

pub fn draw_json(ui: &mut Ui, state: &State) {
    let mut json = String::new();
    for event in state
        .events
        .get_events_list(events_state::DisplayOptions::ALL)
    {
        let rv = serde_json::to_string_pretty(&event.into_serializable()).unwrap();
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        egui_extras::syntax_highlighting::code_view_ui(ui, &theme, rv.as_str(), "json");
        ui.separator();
    }
}
