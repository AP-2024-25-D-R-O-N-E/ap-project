use egui::{ScrollArea, Ui};
use egui_extras::{Column, Table, TableBuilder};

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_section_console(ui: &mut Ui, state: &mut State) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            //TODO draw node events
            // ui.label(egui::RichText::new("This is red text!").color(egui::Color32::LIGHT_GRAY));
            let events = state.events.get_events_list(DisplayOptions::ALL);
            // println!("{:?}", events.len());
            //
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("First column");
                    });
                    header.col(|ui| {
                        ui.heading("Second column");
                    });
                })
                .body(|mut body| {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Hello");
                        });
                        row.col(|ui| {
                            ui.button("world!");
                        });
                    });
                });
            // let mut table = TableBuilder::new(ui)
            //     .striped(self.striped)
            //     .resizable(self.resizable)
            //     .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            //     .column(Column::auto())
            //     .column(
            //         Column::remainder()
            //             .at_least(40.0)
            //             .clip(true)
            //             .resizable(true),
            //     )
            //     .column(Column::auto())
            //     .column(Column::remainder())
            //     .column(Column::remainder())
            //     .min_scrolled_height(0.0)
            //     .max_scroll_height(available_height);
            //
            // egui::Grid::new("my_grid")
            //     .num_columns(2)
            //     .spacing([40.0, 4.0])
            //     .striped(true)
            //     .show(ui, |ui| {
            //         for event in state.events.get_events_list(DisplayOptions::ALL) {
            //             // ui.label(egui::RichText::new(format!("{:?}", event)).color(egui::Color32::LIGHT_GRAY));
            //             event.draw(ui, state)
            //         }
            //     });

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
