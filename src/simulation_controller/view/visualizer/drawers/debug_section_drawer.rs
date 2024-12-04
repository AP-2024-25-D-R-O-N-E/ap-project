use egui::Ui;

use crate::simulation_controller::state::State;

pub fn draw_section_debug(ui: &mut Ui, state: &mut State) {
    ui.label(format!("zoom: {:.5}", state.debug_section.zoom));
    ui.label(format!(
        "pan: [{:.5}, {:.5}]",
        state.debug_section.pan[0], state.debug_section.pan[1]
    ));
    ui.label(format!("FPS: {:.1}", state.debug_section.fps));
}
