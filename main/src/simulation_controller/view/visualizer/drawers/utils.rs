use egui::{Label, RichText, Ui};

use crate::simulation_controller::util::{colors, StatusFlag};

pub fn display_status_flag(status_flag: &StatusFlag, ui: &mut Ui) {
    if let Some(status) = &status_flag {
        match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        }
    }
}

#[allow(unused)]
pub fn get_status_flag_widget(status_flag: &StatusFlag) -> Label {
    match status_flag {
        Some(Ok(s)) => Label::new(RichText::new(s).color(colors::MUTED_GREEN)),
        Some(Err(s)) => Label::new(RichText::new(s).color(colors::MUTED_RED)),
        None => Label::new(RichText::new("")),
    }
}

pub fn get_result_flag_widget<T: ToString, U: ToString>(status_flag: &Result<T, U>) -> Label {
    match status_flag {
        Ok(s) => Label::new(RichText::new(s.to_string()).color(colors::MUTED_GREEN)),
        Err(s) => Label::new(RichText::new(s.to_string()).color(colors::MUTED_RED)),
    }
}
