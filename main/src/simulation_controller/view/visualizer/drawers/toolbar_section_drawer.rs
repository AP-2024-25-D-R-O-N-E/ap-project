use egui::{Layout, Response, Ui, Widget};

use crate::simulation_controller::{
    node::{UiDroneNode, UiNodePayload},
    serialization_ref_structs::IntoSerializable,
    state::{DisplayOptions, State},
    util,
};

use super::{display_status_flag, get_result_flag_widget, get_status_flag_widget, toggle_compact};

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

        // Toggle button: modify topology
        if ui
            .selectable_label(state.toolbar_section.modify_topology_open, "Topology")
            .clicked()
        {
            state.toolbar_section.modify_topology_open =
                !state.toolbar_section.modify_topology_open;
        }

        ui.separator();

        if ui
            .selectable_label(state.toolbar_section.events_json_open, "Events json")
            .clicked()
        {
            state.toolbar_section.events_json_open = !state.toolbar_section.events_json_open;
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

        ui.add_space(30.0);
        // Regular button: Close all
        if ui.button("Close all").clicked() {
            state.node_info_section.opened_windows.clear();
        }

        // Regular button: Open selected
        if ui.button("Open selected").clicked() {
            let selected_nodes = state.graph_section.g.selected_nodes().to_owned();
            for node_index in selected_nodes {
                state.node_info_section.opened_windows.insert(node_index);
            }
        }

        if ui.button("Clear events").clicked() {
            state.events.events.clear();
        }

        ui.add_space(30.0);

        toggle_compact::toggle_ui_compact(ui, &mut state.toolbar_section.handle_shortcuts);
        ui.label("Handle shortcuts");

        ui.separator();

        ui.add(get_result_flag_widget(
            &state.graph_section.well_formedness_flag,
        ))
    });
}
