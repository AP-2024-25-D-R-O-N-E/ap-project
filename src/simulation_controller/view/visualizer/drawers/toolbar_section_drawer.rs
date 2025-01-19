use egui::Ui;

use crate::simulation_controller::state::State;

//Emojis 🧪📋🛠️❌📂
pub fn draw_toolbar_section(ui: &mut Ui, state: &mut State) {
    ui.horizontal(|ui| {
        // Toggle button: Test
        if ui
            .selectable_label(state.toolbar_section.test_open, "Test")
            .clicked()
        {
            state.toolbar_section.test_open = !state.toolbar_section.test_open;
        }

        ui.separator();

        // Toggle button: Console
        if ui
            .selectable_label(state.toolbar_section.console_open, "Console")
            .clicked()
        {
            state.toolbar_section.console_open = !state.toolbar_section.console_open;
        }

        ui.separator();

        // Toggle button: Settings
        if ui
            .selectable_label(state.toolbar_section.settings_open, "Settings")
            .clicked()
        {
            state.toolbar_section.settings_open = !state.toolbar_section.settings_open;
        }

        ui.separator();

        // Toggle button: Debug
        if ui
            .selectable_label(state.toolbar_section.debug_open, "Debug")
            .clicked()
        {
            state.toolbar_section.debug_open = !state.toolbar_section.debug_open;
        }

        ui.separator();
        // Group for "Close all" and "Open selected"
        // Regular button: Close all
        if ui.button("Close all").clicked() {
            // Handle "Close all" button click
            println!("Close all clicked");
        }

        // Regular button: Open selected
        if ui.button("Open selected").clicked() {
            // Handle "Open selected" button click
            println!("Open selected clicked");
        }
    });
}
