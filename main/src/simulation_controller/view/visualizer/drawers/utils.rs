use egui::{RichText, Ui};

use crate::simulation_controller::util::{colors, StatusFlag};

pub fn display_status_flag(status_flag: &StatusFlag, ui: &mut Ui) {
    match &status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        },
        None => (),
    }
}
