use egui::{ScrollArea, Ui};

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_section_console(ui: &mut Ui, state: &mut State) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            //TODO draw node events
            // ui.label(egui::RichText::new("This is red text!").color(egui::Color32::LIGHT_GRAY));
            let events = state.events.get_events_list(DisplayOptions::ALL);
            // println!("{:?}", events.len());
            for x in state.events.get_events_list(DisplayOptions::ALL) {
                ui.label(egui::RichText::new(format!("{:?}", x)).color(egui::Color32::LIGHT_GRAY));
            }

            // ui.label(
            //     egui::RichText::new("This is green bold text!")
            //         .color(egui::Color32::LIGHT_GRAY)
            //         .strong(),
            // );
            //
            // ui.label(
            //     egui::RichText::new("This is blue italic text!")
            //         .color(egui::Color32::LIGHT_GRAY)
            //         .italics(),
            // );
        });
}
