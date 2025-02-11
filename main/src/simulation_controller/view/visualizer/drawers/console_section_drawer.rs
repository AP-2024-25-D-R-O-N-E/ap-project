use egui::{ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_section_console(ui: &mut Ui, state: &mut State) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let text_height = ui.text_style_height(&egui::TextStyle::Body);
            TableBuilder::new(ui)
                .column(Column::initial(80.).resizable(true))
                .column(Column::initial(80.).resizable(true))
                .column(Column::initial(80.).resizable(true))
                .column(Column::remainder())
                .striped(true)
                .header(text_height, |mut header| {
                    header.col(|ui| {
                        ui.label("Sender");
                    });
                    header.col(|ui| {
                        ui.label("Type");
                    });
                    header.col(|ui| {
                        ui.label("Session id");
                    });
                    header.col(|ui| {
                        ui.label("Other infos");
                    });
                })
                .body(|mut body| {
                    if !state.toolbar_section.loggin_enabled {
                        return;
                    }
                    let mut hover_index = None;
                    for (index, event) in state
                        .events
                        .get_events_list(DisplayOptions::ALL)
                        .iter()
                        .enumerate()
                    {
                        body.row(30.0, |mut row| {
                            match state.console_section.hovered_row {
                                Some(hovered_index) => {
                                    if index == hovered_index {
                                        row.set_selected(true);
                                    }
                                }
                                None => row.set_selected(false),
                            }

                            event.draw(&mut row);
                            if row.response().contains_pointer() {
                                hover_index = Some(index);
                            }
                        });
                    }
                    state.console_section.hovered_row = hover_index;
                });
        });
}
