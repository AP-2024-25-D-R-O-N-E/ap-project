use egui::{CollapsingHeader, ScrollArea, Ui};

use crate::simulation_controller::state::State;

pub fn draw_section_debug(ui: &mut Ui, state: &mut State) {
    CollapsingHeader::new("Infos")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(format!("zoom: {:.5}", state.debug_section.zoom));
            ui.label(format!(
                "pan: [{:.5}, {:.5}]",
                state.debug_section.pan[0], state.debug_section.pan[1]
            ));
            ui.label(format!("FPS: {:.1}", state.debug_section.fps));
        });

    CollapsingHeader::new("Graph events")
        .default_open(true)
        .show(ui, |ui| {
            if ui.button("clear").clicked() {
                state.debug_section.graph_events.clear();
            }
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    state
                        .debug_section
                        .graph_events
                        .iter()
                        .rev()
                        .for_each(|event| {
                            ui.label(event);
                        });
                });
        });
}
